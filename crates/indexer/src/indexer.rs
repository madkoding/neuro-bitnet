//! Code indexer for processing files and directories

use std::path::Path;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::analyzer::{CodeAnalyzer, TreeSitterAnalyzer};
use crate::chunk::CodeChunk;
use crate::error::{IndexerError, Result};
use crate::languages::Language;

/// Configuration for the code indexer
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Maximum file size to index (in bytes)
    pub max_file_size: usize,
    /// Directories to skip
    pub skip_dirs: Vec<String>,
    /// File patterns to skip
    pub skip_patterns: Vec<String>,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            max_file_size: 1024 * 1024, // 1MB
            skip_dirs: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".next".to_string(),
            ],
            skip_patterns: vec![
                ".min.js".to_string(),
                ".bundle.js".to_string(),
                ".lock".to_string(),
            ],
        }
    }
}

/// Code indexer for processing source files
pub struct CodeIndexer {
    config: IndexerConfig,
}

impl CodeIndexer {
    /// Create a new indexer with default configuration
    pub fn new() -> Self {
        Self {
            config: IndexerConfig::default(),
        }
    }

    /// Create an indexer with custom configuration
    pub fn with_config(config: IndexerConfig) -> Self {
        Self { config }
    }

    /// Index a single file
    pub fn index_file(&self, path: &Path, language: Language) -> Result<Vec<CodeChunk>> {
        if !path.exists() {
            return Err(IndexerError::FileNotFound(path.display().to_string()));
        }

        let metadata = std::fs::metadata(path)?;
        if metadata.len() as usize > self.config.max_file_size {
            warn!("Skipping large file: {:?} ({} bytes)", path, metadata.len());
            return Ok(Vec::new());
        }

        let source = std::fs::read_to_string(path)?;
        let file_path = path.display().to_string();

        debug!("Indexing file: {} ({})", file_path, language);

        let analyzer = TreeSitterAnalyzer::new(language)?;
        analyzer.analyze(&source, &file_path)
    }

    /// Index a single file, auto-detecting language
    pub fn index_file_auto(&self, path: &Path) -> Result<Vec<CodeChunk>> {
        let language = Language::from_path(path)
            .ok_or_else(|| IndexerError::UnsupportedLanguage(path.display().to_string()))?;
        
        self.index_file(path, language)
    }

    /// Index all supported files in a directory
    pub fn index_directory(&self, path: &Path) -> Result<Vec<CodeChunk>> {
        if !path.exists() {
            return Err(IndexerError::FileNotFound(path.display().to_string()));
        }

        info!("Indexing directory: {:?}", path);

        let mut all_chunks = Vec::new();
        let mut file_count = 0;
        let mut error_count = 0;

        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.should_skip(e))
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                    continue;
                }
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            
            // Check if we support this file type
            if Language::from_path(file_path).is_none() {
                continue;
            }

            // Check skip patterns
            let path_str = file_path.display().to_string();
            if self.config.skip_patterns.iter().any(|p| path_str.contains(p)) {
                continue;
            }

            match self.index_file_auto(file_path) {
                Ok(chunks) => {
                    file_count += 1;
                    all_chunks.extend(chunks);
                }
                Err(e) => {
                    warn!("Failed to index {:?}: {}", file_path, e);
                    error_count += 1;
                }
            }
        }

        info!(
            "Indexed {} files, {} chunks, {} errors",
            file_count,
            all_chunks.len(),
            error_count
        );

        Ok(all_chunks)
    }

    fn should_skip(&self, entry: &walkdir::DirEntry) -> bool {
        let file_name = entry.file_name().to_string_lossy();
        
        // Skip hidden files/directories (but not the root we're indexing)
        if entry.depth() > 0 && file_name.starts_with('.') && file_name != "." {
            return true;
        }

        // Skip configured directories
        if entry.file_type().is_dir() {
            return self.config.skip_dirs.iter().any(|d| file_name == *d);
        }

        false
    }

    /// Get statistics about indexed chunks
    pub fn chunk_stats(chunks: &[CodeChunk]) -> ChunkStats {
        use std::collections::HashMap;

        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_file: HashMap<String, usize> = HashMap::new();
        let mut total_lines = 0;

        for chunk in chunks {
            *by_type.entry(chunk.symbol_type.to_string()).or_default() += 1;
            *by_file.entry(chunk.file_path.clone()).or_default() += 1;
            total_lines += chunk.line_count();
        }

        ChunkStats {
            total_chunks: chunks.len(),
            total_lines,
            by_type,
            file_count: by_file.len(),
        }
    }
}

impl Default for CodeIndexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about indexed code chunks
#[derive(Debug)]
pub struct ChunkStats {
    pub total_chunks: usize,
    pub total_lines: usize,
    pub by_type: std::collections::HashMap<String, usize>,
    pub file_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_index_python_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.py");
        
        fs::write(&file_path, r#"
def hello():
    pass

class World:
    pass
"#).unwrap();

        let indexer = CodeIndexer::new();
        let chunks = indexer.index_file(&file_path, Language::Python).unwrap();
        
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_index_directory() {
        let dir = tempdir().unwrap();
        
        // Create test files with proper Python/Rust code
        let py_content = r#"
def main():
    pass

def helper():
    return 1
"#;
        let rs_content = r#"
fn lib() {}

fn another() {}
"#;
        let module_content = r#"
class Module:
    pass

def module_func():
    pass
"#;

        fs::write(dir.path().join("main.py"), py_content).unwrap();
        fs::write(dir.path().join("lib.rs"), rs_content).unwrap();
        
        // Create a subdirectory with more files
        let sub = dir.path().join("src");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("module.py"), module_content).unwrap();

        eprintln!("Test directory: {:?}", dir.path());
        
        // List files in directory
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            let entry = entry.unwrap();
            eprintln!("Found file: {:?}", entry.path());
        }

        let indexer = CodeIndexer::new();
        let chunks = indexer.index_directory(dir.path()).unwrap();
        
        eprintln!("Total chunks found: {}", chunks.len());
        
        // Debug: print what we found
        for chunk in &chunks {
            eprintln!("Found chunk: {} ({:?}) in {}", chunk.name, chunk.symbol_type, chunk.file_path);
        }
        
        // At minimum we should have some chunks from the files
        assert!(!chunks.is_empty(), "Should have at least some chunks from the test files");
    }

    #[test]
    fn test_skip_directories() {
        let dir = tempdir().unwrap();
        
        // Create a file that should be indexed
        fs::write(dir.path().join("main.py"), "def main(): pass").unwrap();
        
        // Create node_modules (should be skipped)
        let node_modules = dir.path().join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        fs::write(node_modules.join("package.js"), "function x() {}").unwrap();

        let indexer = CodeIndexer::new();
        let chunks = indexer.index_directory(dir.path()).unwrap();
        
        // Should only have the main.py chunk, not the node_modules one
        assert!(chunks.iter().all(|c| !c.file_path.contains("node_modules")));
    }

    #[test]
    fn test_auto_language_detection() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        
        fs::write(&file_path, "fn hello() {}").unwrap();

        let indexer = CodeIndexer::new();
        let chunks = indexer.index_file_auto(&file_path).unwrap();
        
        assert!(!chunks.is_empty());
    }
}
