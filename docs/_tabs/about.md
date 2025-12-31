---
layout: page
icon: fas fa-info-circle
order: 2
---

# About neuro-bitnet

**neuro-bitnet** is a high-performance RAG (Retrieval Augmented Generation) server written in Rust with BitNet 1.58-bit local inference support.

## Why neuro-bitnet?

- **Pure Rust** - No Python dependencies, single static binary
- **BitNet Models** - Uses Microsoft's efficient 1.58-bit quantized models
- **CPU-Only** - No GPU required, runs on any modern processor
- **RAG Built-in** - Semantic search and document indexing included
- **Web Search** - Wikipedia integration for knowledge augmentation

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Embeddings | fastembed |
| Inference | bitnet.cpp (subprocess) |
| HTTP Server | Axum |
| CLI | Clap |

## License

This project is dual-licensed under MIT and Apache 2.0.

## Links

- [GitHub Repository](https://github.com/madkoding/neuro-bitnet)
- [Releases](https://github.com/madkoding/neuro-bitnet/releases)
- [Issues](https://github.com/madkoding/neuro-bitnet/issues)
