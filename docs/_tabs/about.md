---
layout: page
icon: fas fa-info-circle
order: 2
---

# About neuro-bitnet / Acerca de neuro-bitnet

---

## 🌟 Project Overview / Descripción del Proyecto

**neuro-bitnet** is a high-performance RAG (Retrieval Augmented Generation) server written entirely in Rust, featuring local inference with Microsoft's revolutionary BitNet 1.58-bit models. The project provides a complete ecosystem for AI-powered document search, code analysis, and natural language processing — all running locally on your CPU.

**neuro-bitnet** es un servidor RAG (Retrieval Augmented Generation) de alto rendimiento escrito completamente en Rust, con inferencia local usando los revolucionarios modelos BitNet 1.58-bit de Microsoft. El proyecto proporciona un ecosistema completo para búsqueda de documentos con IA, análisis de código y procesamiento de lenguaje natural — todo ejecutándose localmente en tu CPU.

---

## 🎯 Why neuro-bitnet? / ¿Por qué neuro-bitnet?

| Feature | Benefit |
|---------|---------|
| **Pure Rust** | No Python dependencies, single static binary / Sin dependencias de Python, binario estático único |
| **BitNet Models** | Microsoft's efficient 1.58-bit quantized models / Modelos cuantizados eficientes de 1.58-bit de Microsoft |
| **CPU-Only** | No GPU required, runs on any modern processor / No requiere GPU, funciona en cualquier procesador moderno |
| **RAG Built-in** | Semantic search and document indexing included / Búsqueda semántica e indexación de documentos incluidos |
| **Web Search** | Wikipedia integration for knowledge augmentation / Integración con Wikipedia para aumentar conocimiento |
| **Multilingual** | Auto-translation for Spanish queries (56% → 100% accuracy) / Auto-traducción para consultas en español |
| **IDE Integration** | MCP server for VS Code and other IDEs / Servidor MCP para VS Code y otros IDEs |

---

## 🛠️ Tech Stack / Stack Tecnológico

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Language** | Rust | Systems programming, memory safety |
| **Inference** | bitnet.cpp | 1.58-bit model execution |
| **Embeddings** | fastembed | Local embedding generation |
| **HTTP Server** | Axum | High-performance async web framework |
| **CLI** | Clap | Command-line argument parsing |
| **Code Parsing** | tree-sitter | Multi-language syntax analysis |
| **Vector Ops** | ndarray | SIMD-optimized similarity search |

---

## 📦 Components / Componentes

### Three Main Binaries / Tres Binarios Principales

| Binary | Description | Use Case |
|--------|-------------|----------|
| **`neuro`** | CLI for immediate command execution | Scripts, one-off queries, model management |
| **`neuro-daemon`** | Background HTTP server with OpenAI-compatible API | Applications, integrations, continuous service |
| **`neuro-mcp`** | Model Context Protocol server | IDE integration (VS Code, etc.) |

### 13 Rust Crates / 13 Crates de Rust

The project is organized as a Cargo workspace with modular crates:

- `neuro-core` - Shared types (Document, SearchResult, QueryCategory)
- `neuro-embeddings` - Embedding generation with fastembed
- `neuro-storage` - Document storage (memory and file-based)
- `neuro-classifier` - Query classification with regex patterns
- `neuro-indexer` - Code indexing with tree-sitter
- `neuro-search` - Web search (Wikipedia integration)
- `neuro-inference` - BitNet inference (native FFI + subprocess)
- `neuro-llm` - OpenAI-compatible client
- `bitnet-sys` - Low-level FFI bindings to bitnet.cpp
- `neuro-server` - Axum HTTP server (RAG API)
- `neuro-cli` - Command-line interface
- `neuro-daemon` - Background HTTP server for inference
- `neuro-mcp` - Model Context Protocol server

---

## 📊 Performance / Rendimiento

| Operation | Time |
|-----------|------|
| Embedding (384 dims) | ~5ms |
| Similarity search (10k docs) | ~2ms |
| Query classification | ~0.1ms |
| Document indexing | ~10ms |
| BitNet inference (simple query) | ~800ms |
| BitNet inference (complex query) | ~2.8s |

---

## 📜 License / Licencia

This project is **dual-licensed** under:

Este proyecto tiene **licencia dual** bajo:

- **[MIT License](https://github.com/madkoding/neuro-bitnet/blob/main/LICENSE-MIT)** - Permissive open source license
- **[Apache License 2.0](https://github.com/madkoding/neuro-bitnet/blob/main/LICENSE-APACHE)** - Patent protection included

You may choose either license at your option.

Puedes elegir cualquiera de las licencias a tu elección.

---

## 👤 Author / Autor

**madkoding**

- GitHub: [@madkoding](https://github.com/madkoding)
- Repository: [neuro-bitnet](https://github.com/madkoding/neuro-bitnet)

---

## 🔗 Links / Enlaces

| Resource | Link |
|----------|------|
| **GitHub Repository** | [github.com/madkoding/neuro-bitnet](https://github.com/madkoding/neuro-bitnet) |
| **Releases** | [Latest Releases](https://github.com/madkoding/neuro-bitnet/releases) |
| **Issues** | [Report a Bug](https://github.com/madkoding/neuro-bitnet/issues) |
| **Crates.io** | [neuro-cli](https://crates.io/crates/neuro-cli) |
| **Documentation** | [GitHub Pages](https://madkoding.github.io/neuro-bitnet/) |

---

## 🙏 Acknowledgments / Agradecimientos

- [Microsoft BitNet](https://github.com/microsoft/BitNet) - Revolutionary 1.58-bit model architecture
- [fastembed](https://github.com/Anush008/fastembed-rs) - Native embedding models in Rust
- [Axum](https://github.com/tokio-rs/axum) - Ergonomic web framework
- [tree-sitter](https://tree-sitter.github.io/tree-sitter/) - Incremental parsing library
- [ndarray](https://github.com/rust-ndarray/ndarray) - N-dimensional arrays for Rust
