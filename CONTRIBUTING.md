# Contributing to neuro-bitnet

Thank you for your interest in contributing to neuro-bitnet! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)

---

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. Please:

- Be respectful and considerate in all interactions
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Accept responsibility for mistakes and learn from them

---

## Getting Started

### Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **Clang 18+** - Required for BitNet compilation
- **CMake 3.14+** - Build system for bitnet.cpp
- **Git** - Version control

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/neuro-bitnet.git
   cd neuro-bitnet
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/madkoding/neuro-bitnet.git
   ```

---

## Development Setup

### Build the Project

```bash
# Development build
cargo build

# Release build
cargo build --release

# Build specific crate
cargo build -p neuro-cli
```

### Setup BitNet (for inference testing)

```bash
./scripts/setup_bitnet.sh
neuro model download 2b
```

### Run Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p neuro-core

# With output
cargo test -- --nocapture
```

### Run Linters

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

---

## Making Changes

### Branch Naming

Use descriptive branch names:

- `feature/add-new-embedding-model`
- `fix/daemon-crash-on-startup`
- `docs/update-api-reference`
- `refactor/improve-storage-performance`

### Commit Messages

Follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, etc.)
- `refactor` - Code refactoring
- `test` - Adding or updating tests
- `chore` - Maintenance tasks

Examples:
```
feat(daemon): add auto-translation for Spanish queries
fix(inference): resolve memory leak in context pool
docs(api): update endpoint documentation
```

### Keep Commits Atomic

Each commit should:
- Represent a single logical change
- Be buildable on its own
- Have passing tests

---

## Pull Request Process

### Before Submitting

1. **Sync with upstream:**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks:**
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```

3. **Update documentation** if needed

### PR Guidelines

1. **Title:** Clear, descriptive title following commit convention
2. **Description:** Explain what changes were made and why
3. **Link issues:** Reference related issues with `Fixes #123` or `Relates to #456`
4. **Screenshots:** Include for UI changes
5. **Breaking changes:** Clearly document any breaking changes

### Review Process

1. Maintainers will review your PR
2. Address feedback with additional commits
3. Once approved, maintainers will merge
4. Delete your branch after merge

---

## Code Style

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting

### General Rules

```rust
// Use descriptive names
fn calculate_similarity_score(a: &[f32], b: &[f32]) -> f32 { ... }

// Document public items
/// Generates embeddings for the given text.
///
/// # Arguments
/// * `text` - The input text to embed
///
/// # Returns
/// A vector of f32 values representing the embedding
pub fn embed(&self, text: &str) -> Vec<f32> { ... }

// Handle errors properly
pub fn load_model(path: &Path) -> Result<Model, NeuroError> { ... }

// Use strong types
pub struct DocumentId(Uuid);
```

### Error Handling

- Use `thiserror` for error types
- Provide context in errors
- Don't panic in library code

```rust
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("Model not found at path: {0}")]
    ModelNotFound(PathBuf),
    
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
}
```

---

## Testing

### Test Organization

```
crates/
  core/
    src/
      lib.rs
    tests/          # Integration tests
      integration.rs
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new("content");
        assert!(!doc.id.is_nil());
        assert_eq!(doc.content, "content");
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Test Coverage

- Aim for high test coverage on core logic
- Integration tests for API endpoints
- Document tests with `cargo test --doc`

---

## Documentation

### Code Documentation

- Document all public items
- Include examples in doc comments
- Use `# Examples` section

```rust
/// Classifies a query into a category.
///
/// # Arguments
/// * `query` - The query string to classify
///
/// # Returns
/// The classified category
///
/// # Examples
/// ```
/// use neuro_classifier::classify;
///
/// let category = classify("What is 2 + 2?");
/// assert_eq!(category, QueryCategory::Math);
/// ```
pub fn classify(query: &str) -> QueryCategory { ... }
```

### Website Documentation

Documentation site is in `docs/` using Jekyll:

1. Edit/create markdown files in `docs/_posts/`
2. Follow existing post format
3. Include both English and Spanish versions

---

## Project Structure

```
neuro-bitnet/
├── Cargo.toml              # Workspace configuration
├── crates/                 # Individual crates
│   ├── core/               # Shared types
│   ├── cli/                # CLI application
│   ├── daemon/             # HTTP daemon
│   ├── mcp/                # MCP server
│   ├── inference/          # BitNet inference
│   ├── embeddings/         # Embedding generation
│   ├── storage/            # Document storage
│   ├── classifier/         # Query classification
│   ├── indexer/            # Code indexing
│   ├── search/             # Web search
│   ├── server/             # RAG HTTP server
│   ├── llm/                # LLM client
│   └── bitnet-sys/         # FFI bindings
├── docs/                   # GitHub Pages documentation
├── scripts/                # Setup scripts
├── benchmarks/             # Benchmark scripts
└── vendor/                 # Vendored dependencies
```

---

## Getting Help

- **Issues:** Open a GitHub issue for bugs or features
- **Discussions:** Use GitHub Discussions for questions
- **Documentation:** Check the [docs site](https://madkoding.github.io/neuro-bitnet/)

---

## License

By contributing, you agree that your contributions will be licensed under the same dual license as the project: MIT and Apache 2.0.

---

Thank you for contributing to neuro-bitnet! 🎉
