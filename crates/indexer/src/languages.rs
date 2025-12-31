//! Supported programming languages

use std::path::Path;

/// Supported programming languages for indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Python,
    JavaScript,
    TypeScript,
    Rust,
}

impl Language {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "py" => Some(Self::Python),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "rs" => Some(Self::Rust),
            _ => None,
        }
    }

    /// Detect language from file path
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    /// Get the language name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Rust => "Rust",
        }
    }

    /// Get common file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Python => &["py"],
            Self::JavaScript => &["js", "mjs", "cjs"],
            Self::TypeScript => &["ts", "tsx"],
            Self::Rust => &["rs"],
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "python" | "py" => Ok(Self::Python),
            "javascript" | "js" => Ok(Self::JavaScript),
            "typescript" | "ts" => Ok(Self::TypeScript),
            "rust" | "rs" => Ok(Self::Rust),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_extension() {
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("unknown"), None);
    }

    #[test]
    fn test_from_path() {
        assert_eq!(
            Language::from_path(Path::new("main.py")),
            Some(Language::Python)
        );
        assert_eq!(
            Language::from_path(Path::new("/some/path/file.rs")),
            Some(Language::Rust)
        );
    }

    #[test]
    fn test_from_str() {
        assert_eq!("python".parse::<Language>().unwrap(), Language::Python);
        assert_eq!("js".parse::<Language>().unwrap(), Language::JavaScript);
    }
}
