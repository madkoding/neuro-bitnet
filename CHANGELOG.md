# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive documentation for GitHub Pages
- Bilingual documentation (English/Spanish)
- Daemon server guide with OpenAI-compatible API documentation
- MCP integration guide for VS Code
- Architecture overview documentation
- Translation benchmark results publication
- CONTRIBUTING.md with contribution guidelines
- This CHANGELOG.md file

### Changed
- Redesigned home page with project overview and quick start
- Updated getting-started guide with daemon and MCP sections
- Expanded API reference with daemon endpoints
- Improved about page with full technology stack

## [0.1.0] - 2025-12-31

### Added
- Initial release of neuro-bitnet
- **neuro-cli**: Command-line interface for RAG operations and inference
- **neuro-daemon**: Background HTTP server with OpenAI-compatible API
- **neuro-mcp**: Model Context Protocol server for IDE integration
- **neuro-core**: Shared types (Document, SearchResult, QueryCategory)
- **neuro-embeddings**: Embedding generation via fastembed
- **neuro-storage**: Document storage (memory and file-based)
- **neuro-classifier**: Query classification with regex patterns
- **neuro-indexer**: Code indexing with tree-sitter
- **neuro-search**: Wikipedia integration for web search
- **neuro-inference**: BitNet inference (native FFI and subprocess)
- **neuro-llm**: OpenAI-compatible client
- **neuro-server**: Axum HTTP server for RAG API
- **bitnet-sys**: Low-level FFI bindings to bitnet.cpp
- BitNet 1.58-bit model support (2B, 3B, 8B models)
- Auto-translation for Spanish queries (56% → 100% accuracy improvement)
- Streaming output support
- Web search context integration
- RAG document indexing and search
- Multi-language code parsing (Rust, Python, JavaScript, TypeScript)
- Docker support with docker-compose
- Benchmark scripts and reports

### Performance
- 100% pass rate on factual queries (BitNet 2B)
- ~800ms average response time (native FFI)
- ~2.8s average response time (subprocess)
- CPU-only inference (no GPU required)

---

## Version History

### Versioning Scheme

- **Major (X.0.0)**: Breaking changes to public API
- **Minor (0.X.0)**: New features, backward compatible
- **Patch (0.0.X)**: Bug fixes, backward compatible

### Release Branches

- `main`: Latest stable release
- `develop`: Development branch for next release

---

## Links

- [GitHub Releases](https://github.com/madkoding/neuro-bitnet/releases)
- [Documentation](https://madkoding.github.io/neuro-bitnet/)
- [Issues](https://github.com/madkoding/neuro-bitnet/issues)
