//! Code analyzer trait and implementations

use tree_sitter::{Parser, Tree};
use crate::chunk::{CodeChunk, SymbolType};
use crate::error::{IndexerError, Result};
use crate::languages::Language;

/// Trait for language-specific code analysis
pub trait CodeAnalyzer {
    /// Get the language this analyzer handles
    fn language(&self) -> Language;

    /// Parse source code into chunks
    fn analyze(&self, source: &str, file_path: &str) -> Result<Vec<CodeChunk>>;
}

/// Generic tree-sitter based analyzer
pub struct TreeSitterAnalyzer {
    language: Language,
    parser: Parser,
}

impl TreeSitterAnalyzer {
    /// Create a new analyzer for the given language
    pub fn new(language: Language) -> Result<Self> {
        let mut parser = Parser::new();
        
        let ts_language = match language {
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        };

        parser
            .set_language(&ts_language)
            .map_err(|e| IndexerError::TreeSitter(e.to_string()))?;

        Ok(Self { language, parser })
    }

    fn parse(&mut self, source: &str) -> Result<Tree> {
        self.parser
            .parse(source, None)
            .ok_or_else(|| IndexerError::ParseError("Failed to parse source code".into()))
    }

    fn extract_chunks(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<CodeChunk> {
        let mut chunks = Vec::new();
        let root = tree.root_node();
        
        self.visit_node(root, source, file_path, &mut chunks, None);
        
        chunks
    }

    fn visit_node(
        &self,
        node: tree_sitter::Node,
        source: &str,
        file_path: &str,
        chunks: &mut Vec<CodeChunk>,
        parent: Option<&str>,
    ) {
        let kind = node.kind();
        
        // Check if this node type is interesting for our language
        if let Some((symbol_type, name)) = self.classify_node(&node, source) {
            let start_line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;
            
            let content = node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();

            let mut chunk = CodeChunk::new(
                name.clone(),
                symbol_type,
                content,
                file_path,
                start_line,
                end_line,
            );

            if let Some(p) = parent {
                chunk = chunk.with_parent(p);
            }

            // Try to extract documentation
            if let Some(doc) = self.extract_documentation(&node, source) {
                chunk = chunk.with_documentation(doc);
            }

            // Try to extract signature
            if let Some(sig) = self.extract_signature(&node, source) {
                chunk = chunk.with_signature(sig);
            }

            // For classes/structs, visit children with this as parent
            let new_parent = if matches!(symbol_type, SymbolType::Class | SymbolType::Struct | SymbolType::Impl) {
                Some(name.as_str())
            } else {
                parent
            };

            chunks.push(chunk);

            // Visit children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.visit_node(child, source, file_path, chunks, new_parent);
            }
        } else {
            // Not interesting, but check children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.visit_node(child, source, file_path, chunks, parent);
            }
        }
    }

    fn classify_node(&self, node: &tree_sitter::Node, source: &str) -> Option<(SymbolType, String)> {
        let kind = node.kind();
        
        match self.language {
            Language::Python => self.classify_python_node(node, kind, source),
            Language::JavaScript | Language::TypeScript => self.classify_js_node(node, kind, source),
            Language::Rust => self.classify_rust_node(node, kind, source),
        }
    }

    fn classify_python_node(&self, node: &tree_sitter::Node, kind: &str, source: &str) -> Option<(SymbolType, String)> {
        match kind {
            "function_definition" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Function, name))
            }
            "class_definition" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Class, name))
            }
            _ => None,
        }
    }

    fn classify_js_node(&self, node: &tree_sitter::Node, kind: &str, source: &str) -> Option<(SymbolType, String)> {
        match kind {
            "function_declaration" | "method_definition" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Function, name))
            }
            "class_declaration" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Class, name))
            }
            "arrow_function" => {
                // Try to get name from parent variable declaration
                Some((SymbolType::Function, "anonymous".to_string()))
            }
            _ => None,
        }
    }

    fn classify_rust_node(&self, node: &tree_sitter::Node, kind: &str, source: &str) -> Option<(SymbolType, String)> {
        match kind {
            "function_item" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Function, name))
            }
            "struct_item" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Struct, name))
            }
            "enum_item" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Enum, name))
            }
            "trait_item" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Trait, name))
            }
            "impl_item" => {
                // Get the type being implemented
                let type_node = node.child_by_field_name("type")?;
                let name = type_node.utf8_text(source.as_bytes()).ok()?.to_string();
                Some((SymbolType::Impl, name))
            }
            "mod_item" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Module, name))
            }
            "const_item" | "static_item" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::Constant, name))
            }
            "type_item" => {
                let name = self.get_child_by_field(node, "name", source)?;
                Some((SymbolType::TypeAlias, name))
            }
            _ => None,
        }
    }

    fn get_child_by_field(&self, node: &tree_sitter::Node, field: &str, source: &str) -> Option<String> {
        node.child_by_field_name(field)?
            .utf8_text(source.as_bytes())
            .ok()
            .map(|s| s.to_string())
    }

    fn extract_documentation(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        // Look for preceding comment/docstring
        let prev = node.prev_sibling()?;
        let kind = prev.kind();

        let is_doc = match self.language {
            Language::Python => kind == "expression_statement" || kind == "comment",
            Language::JavaScript | Language::TypeScript => kind == "comment",
            Language::Rust => kind == "line_comment" || kind == "block_comment",
        };

        if is_doc {
            prev.utf8_text(source.as_bytes()).ok().map(|s| s.to_string())
        } else {
            None
        }
    }

    fn extract_signature(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        // Get first line of the node as signature
        let text = node.utf8_text(source.as_bytes()).ok()?;
        let first_line = text.lines().next()?;
        Some(first_line.trim().to_string())
    }
}

impl CodeAnalyzer for TreeSitterAnalyzer {
    fn language(&self) -> Language {
        self.language
    }

    fn analyze(&self, source: &str, file_path: &str) -> Result<Vec<CodeChunk>> {
        // Need mutable self for parsing - create a new parser each time
        let mut parser = Parser::new();
        
        let ts_language = match self.language {
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        };

        parser
            .set_language(&ts_language)
            .map_err(|e| IndexerError::TreeSitter(e.to_string()))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| IndexerError::ParseError("Failed to parse source code".into()))?;

        Ok(self.extract_chunks(&tree, source, file_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_analyzer() {
        let analyzer = TreeSitterAnalyzer::new(Language::Python).unwrap();
        let source = r#"
def hello(name):
    """Greet someone."""
    print(f"Hello, {name}!")

class Greeter:
    def greet(self, name):
        return f"Hello, {name}"
"#;

        let chunks = analyzer.analyze(source, "test.py").unwrap();
        
        // Should find: hello function, Greeter class, greet method
        assert!(chunks.len() >= 2);
        
        let function_names: Vec<_> = chunks.iter()
            .filter(|c| c.symbol_type == SymbolType::Function)
            .map(|c| c.name.as_str())
            .collect();
        
        assert!(function_names.contains(&"hello"));
    }

    #[test]
    fn test_rust_analyzer() {
        let analyzer = TreeSitterAnalyzer::new(Language::Rust).unwrap();
        let source = r#"
/// A simple struct
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

fn main() {
    let p = Point::new(1.0, 2.0);
}
"#;

        let chunks = analyzer.analyze(source, "main.rs").unwrap();
        
        let struct_chunks: Vec<_> = chunks.iter()
            .filter(|c| c.symbol_type == SymbolType::Struct)
            .collect();
        
        assert!(!struct_chunks.is_empty());
        assert_eq!(struct_chunks[0].name, "Point");
    }

    #[test]
    fn test_javascript_analyzer() {
        let analyzer = TreeSitterAnalyzer::new(Language::JavaScript).unwrap();
        let source = r#"
function greet(name) {
    return `Hello, ${name}!`;
}

class Person {
    constructor(name) {
        this.name = name;
    }
}
"#;

        let chunks = analyzer.analyze(source, "app.js").unwrap();
        assert!(!chunks.is_empty());
    }
}
