//! Code chunk representation

use serde::{Deserialize, Serialize};

/// Type of code symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolType {
    /// Function or method
    Function,
    /// Class definition
    Class,
    /// Struct definition (Rust)
    Struct,
    /// Enum definition
    Enum,
    /// Trait/Interface definition
    Trait,
    /// Implementation block (Rust)
    Impl,
    /// Module/namespace
    Module,
    /// Constant or static variable
    Constant,
    /// Type alias
    TypeAlias,
    /// Import statement
    Import,
    /// Other code block
    Other,
}

impl std::fmt::Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Class => write!(f, "class"),
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::Trait => write!(f, "trait"),
            Self::Impl => write!(f, "impl"),
            Self::Module => write!(f, "module"),
            Self::Constant => write!(f, "constant"),
            Self::TypeAlias => write!(f, "type_alias"),
            Self::Import => write!(f, "import"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// A chunk of code extracted from a source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChunk {
    /// Name of the symbol (function name, class name, etc.)
    pub name: String,

    /// Type of symbol
    pub symbol_type: SymbolType,

    /// The actual code content
    pub content: String,

    /// Source file path
    pub file_path: String,

    /// Start line (1-indexed)
    pub start_line: usize,

    /// End line (1-indexed)
    pub end_line: usize,

    /// Parent symbol name (e.g., class name for a method)
    pub parent: Option<String>,

    /// Documentation/docstring if available
    pub documentation: Option<String>,

    /// Function/method signature (if applicable)
    pub signature: Option<String>,
}

impl CodeChunk {
    /// Create a new code chunk
    pub fn new(
        name: impl Into<String>,
        symbol_type: SymbolType,
        content: impl Into<String>,
        file_path: impl Into<String>,
        start_line: usize,
        end_line: usize,
    ) -> Self {
        Self {
            name: name.into(),
            symbol_type,
            content: content.into(),
            file_path: file_path.into(),
            start_line,
            end_line,
            parent: None,
            documentation: None,
            signature: None,
        }
    }

    /// Set parent symbol
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    /// Set documentation
    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }

    /// Set signature
    pub fn with_signature(mut self, sig: impl Into<String>) -> Self {
        self.signature = Some(sig.into());
        self
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }

    /// Get a display string for this chunk
    pub fn display_name(&self) -> String {
        match &self.parent {
            Some(parent) => format!("{}::{}", parent, self.name),
            None => self.name.clone(),
        }
    }

    /// Convert to a document-friendly content string
    pub fn to_document_content(&self) -> String {
        let mut content = String::new();

        // Add metadata header
        content.push_str(&format!(
            "# {} `{}`\n",
            self.symbol_type, self.display_name()
        ));
        content.push_str(&format!("File: {}:{}-{}\n", self.file_path, self.start_line, self.end_line));

        if let Some(ref doc) = self.documentation {
            content.push_str(&format!("\n{}\n", doc));
        }

        if let Some(ref sig) = self.signature {
            content.push_str(&format!("\nSignature: `{}`\n", sig));
        }

        content.push_str("\n```\n");
        content.push_str(&self.content);
        if !self.content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str("```\n");

        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_chunk_creation() {
        let chunk = CodeChunk::new(
            "my_function",
            SymbolType::Function,
            "def my_function():\n    pass",
            "main.py",
            10,
            12,
        );

        assert_eq!(chunk.name, "my_function");
        assert_eq!(chunk.symbol_type, SymbolType::Function);
        assert_eq!(chunk.line_count(), 3);
    }

    #[test]
    fn test_display_name() {
        let chunk = CodeChunk::new("method", SymbolType::Function, "...", "file.py", 1, 5)
            .with_parent("MyClass");

        assert_eq!(chunk.display_name(), "MyClass::method");
    }

    #[test]
    fn test_to_document_content() {
        let chunk = CodeChunk::new(
            "greet",
            SymbolType::Function,
            "fn greet(name: &str) { println!(\"Hello, {}!\", name); }",
            "main.rs",
            5,
            7,
        )
        .with_documentation("Greets a person by name")
        .with_signature("fn greet(name: &str)");

        let content = chunk.to_document_content();
        assert!(content.contains("function"));
        assert!(content.contains("greet"));
        assert!(content.contains("main.rs:5-7"));
        assert!(content.contains("Greets a person"));
    }
}
