# neuro-bitnet

[![CI](https://github.com/madkoding/neuro-bitnet/actions/workflows/ci.yml/badge.svg)](https://github.com/madkoding/neuro-bitnet/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/neuro-cli.svg)](https://crates.io/crates/neuro-cli)
[![License](https://img.shields.io/crates/l/neuro-cli.svg)](LICENSE-MIT)

🌐 **[Documentation](https://madkoding.github.io/neuro-bitnet/)** | **[Documentación en Español](https://madkoding.github.io/neuro-bitnet/es/)**

A high-performance **RAG (Retrieval Augmented Generation)** server written in Rust with **BitNet 1.58-bit** local inference. Features intelligent query classification, native embeddings, and CPU-only inference using Microsoft's BitNet models.

## ✨ Features

- 🚀 **High Performance** - Native Rust with SIMD-optimized vector operations
- 🧠 **BitNet Inference** - Local CPU-only inference with Microsoft's 1.58-bit models
- 📊 **Native Embeddings** - Built-in embedding models via fastembed (no external services)
- 🔍 **Semantic Search** - Fast cosine similarity search with ndarray
- 🌐 **Web Search** - Wikipedia integration for knowledge augmentation
- 🛠️ **Code Analysis** - Tree-sitter powered multi-language parsing
- 📦 **Single Binary** - Static compilation, no runtime dependencies

## 🧠 BitNet Local Inference

neuro-bitnet supports local inference using Microsoft's BitNet 1.58-bit models. No GPU required!

```bash
# Setup BitNet (one time)
./scripts/setup_bitnet.sh

# Download a model
neuro model download 2b

# Ask questions locally
neuro ask "What is the capital of France?"
```

### Benchmark Results

| Metric | BitNet b1.58 2B-4T |
|--------|-------------------|
| **Pass Rate** | 100% |
| **Model Size** | 1.1 GB |
| **Avg Response** | 2.8s |
| **Backend** | CPU-only |

[See full benchmark report →](https://madkoding.github.io/neuro-bitnet/benchmarks)

## 🚀 Installation

### Pre-built Binaries

Download the latest release for your platform:

```bash
# Linux x86_64
curl -L https://github.com/madkoding/neuro-bitnet/releases/latest/download/neuro-linux-x86_64 -o neuro
chmod +x neuro
sudo mv neuro /usr/local/bin/

# Linux aarch64
curl -L https://github.com/madkoding/neuro-bitnet/releases/latest/download/neuro-linux-aarch64 -o neuro
chmod +x neuro
sudo mv neuro /usr/local/bin/

# macOS x86_64
curl -L https://github.com/madkoding/neuro-bitnet/releases/latest/download/neuro-darwin-x86_64 -o neuro
chmod +x neuro
sudo mv neuro /usr/local/bin/

# macOS Apple Silicon
curl -L https://github.com/madkoding/neuro-bitnet/releases/latest/download/neuro-darwin-aarch64 -o neuro
chmod +x neuro
sudo mv neuro /usr/local/bin/
```

### Cargo Install

```bash
cargo install neuro-cli

# With CUDA support
cargo install neuro-cli --features cuda
```

### Build from Source

```bash
git clone https://github.com/madkoding/neuro-bitnet.git
cd neuro-bitnet

# Development build
cargo build

# Release build (optimized, static binary)
cargo build --release --target x86_64-unknown-linux-musl
```

## 📖 Usage

### CLI Commands

```bash
# Start the HTTP server
neuro serve --port 8080

# Start with persistent storage
neuro serve --port 8080 --storage ./data

# Index a directory
neuro index ./src --recursive --include "*.rs"

# Execute a query
neuro query "What is Rust?" --storage ./data

# Show storage statistics
neuro stats --storage ./data

# Classify a query
neuro classify "Calculate 2 + 2"

# Generate embeddings
neuro embed "Hello world"

# Search Wikipedia
neuro search "Rust programming language"

# Ask a question (requires BitNet/llama.cpp server)
neuro ask "What is the capital of France?"

# Ask with web search context
neuro ask "Explain quantum computing" --web

# Ask with RAG context
neuro ask "Summarize the code" --storage ./data --timing

# Ask in Spanish (auto-translate)
neuro ask "¿Qué es la programación funcional?" --translate
```

### Daemon Server

Run a background inference server with OpenAI-compatible API:

```bash
# Start the daemon (default port 11435)
neuro-daemon --foreground

# With custom port and model
neuro-daemon --port 8000 --model /path/to/model.gguf --foreground

# Disable auto-translation
neuro-daemon --auto-translate false --foreground

# Use as systemd service (daemonize)
neuro-daemon --pid-file /var/run/neuro-daemon.pid
```

#### Daemon API

```bash
# Health check
curl http://localhost:11435/health

# Generate text (OpenAI-compatible)
curl -X POST http://localhost:11435/v1/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "What is Rust?", "max_tokens": 256}'

# Chat completion (OpenAI-compatible)
curl -X POST http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "messages": [{"role": "user", "content": "Explain quantum computing"}],
    "max_tokens": 512
  }'

# Spanish query (auto-translated)
curl -X POST http://localhost:11435/v1/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "¿Cuál es la capital de Francia?"}'
```

### MCP Server (IDE Integration)

The MCP (Model Context Protocol) server enables IDE integration:

```bash
# Run MCP server (uses stdio transport)
neuro-mcp --model /path/to/model.gguf

# With debug logging
neuro-mcp --debug
```

#### VS Code Configuration

Add to your VS Code settings (`.vscode/settings.json`):

```json
{
  "mcpServers": {
    "neuro-bitnet": {
      "command": "neuro-mcp",
      "args": ["--model", "/path/to/model.gguf"]
    }
  }
}
```

#### Available MCP Tools

| Tool | Description |
|------|-------------|
| `generate` | Generate text from a prompt (supports Spanish) |
| `translate` | Translate text to English |
| `ask` | Ask a question with optional context |
| `summarize` | Summarize text |

### HTTP API

```bash
# Health check
curl http://localhost:8080/health

# Get statistics
curl http://localhost:8080/stats

# Add a document
curl -X POST http://localhost:8080/add \
  -H "Content-Type: application/json" \
  -d '{"content": "Rust is a systems programming language"}'

# Search documents
curl -X POST http://localhost:8080/search \
  -H "Content-Type: application/json" \
  -d '{"query": "programming language", "top_k": 5}'

# Classify a query
curl -X POST http://localhost:8080/classify \
  -H "Content-Type: application/json" \
  -d '{"query": "What is 2 + 2?"}'

# Execute an intelligent query
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is Rust?", "top_k": 5}'

# List all documents
curl http://localhost:8080/documents
```

## 🏗️ Architecture

```
neuro-bitnet/
├── crates/
│   ├── core/         # Shared types (Document, SearchResult, etc.)
│   ├── embeddings/   # fastembed-based embedding generation
│   ├── storage/      # Document storage (memory, file-based)
│   ├── classifier/   # Query classification with regex patterns
│   ├── indexer/      # Code analysis with tree-sitter
│   ├── search/       # Web search (Wikipedia integration)
│   ├── inference/    # BitNet inference (native FFI + subprocess)
│   ├── bitnet-sys/   # Low-level FFI bindings to bitnet.cpp
│   ├── server/       # Axum HTTP server (RAG API)
│   ├── cli/          # Command-line interface (immediate execution)
│   ├── daemon/       # Background HTTP server for inference
│   └── mcp/          # Model Context Protocol server for IDE integration
```

### Components

- **CLI (`neuro`)**: Immediate command execution for queries, indexing, and model management
- **Daemon (`neuro-daemon`)**: Background HTTP server with OpenAI-compatible API
- **MCP Server (`neuro-mcp`)**: IDE integration via Model Context Protocol (VS Code, etc.)

### Multilingual Support

All components support queries in Spanish (auto-translated to English):

```bash
# CLI with translation
neuro ask "¿Cuál es la capital de Francia?" --translate

# Daemon auto-translates
curl -X POST http://localhost:11435/v1/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "¿Qué es Rust?"}'
```

## 🔗 Native Bindings

neuro-bitnet supports two inference backends for BitNet models:

### Backend Types

| Backend | Feature Flag | Description | Performance |
|---------|--------------|-------------|-------------|
| **Native FFI** | `--features native` | Direct bindings to bitnet.cpp | ~50-70% faster |
| **Subprocess** | `--features subprocess` (default) | Calls `llama-cli` binary | Works out of the box |

### Building with Native Bindings

Native bindings compile bitnet.cpp from source for maximum performance:

```bash
# Requirements (Linux)
sudo apt-get install cmake clang libstdc++-12-dev

# Clone with submodules
git clone --recursive https://github.com/madkoding/neuro-bitnet.git
cd neuro-bitnet

# Initialize BitNet submodule (if not cloned with --recursive)
git submodule update --init --recursive
cd vendor/BitNet && git submodule update --init --recursive && cd ../..

# Build with native bindings
cargo build --release --features native

# Or use auto-selection (tries native first, falls back to subprocess)
cargo build --release
```

### Selecting Backend at Runtime

```rust
use neuro_inference::{InferenceConfig, InferenceModel, BackendType};

// Auto-select best backend
let config = InferenceConfig::new("model.gguf");
let model = InferenceModel::load(config)?;

// Force specific backend
let config = InferenceConfig::new("model.gguf")
    .with_backend(BackendType::Native);
let model = InferenceModel::load(config)?;
```

### Performance Comparison

| Backend | Avg Response Time | Notes |
|---------|-------------------|-------|
| Subprocess | ~2800ms | Creates new process per request |
| Native FFI | ~800-1200ms | Reuses context pool, direct FFI |

Native bindings use a context pool for concurrent requests, reducing latency significantly for high-throughput scenarios.

### Query Categories

The classifier automatically categorizes queries:

| Category | Description | Strategy |
|----------|-------------|----------|
| `math` | Mathematical calculations | Direct computation |
| `code` | Programming questions | RAG search |
| `reasoning` | Logic and analysis | RAG then LLM |
| `tools` | Tool/function requests | Tool calling |
| `greeting` | Casual greetings | Direct response |
| `factual` | Factual questions | RAG then Web |
| `conversational` | General conversation | RAG search |

### Embedding Models

Supported models via fastembed:

| Model | Dimensions | Size | Speed |
|-------|------------|------|-------|
| `minilm` (default) | 384 | ~90MB | Fast |
| `bge-small` | 384 | ~133MB | Fast |
| `bge-base` | 768 | ~436MB | Medium |
| `bge-large` | 1024 | ~1.3GB | Slow |
| `gte-small` | 384 | ~67MB | Fast |
| `gte-base` | 768 | ~219MB | Medium |
| `e5-small` | 384 | ~133MB | Fast |
| `e5-base` | 768 | ~436MB | Medium |
| `e5-large` | 1024 | ~1.3GB | Slow |

## ⚙️ Configuration

### Environment Variables

```bash
# Server configuration
NEURO_HOST=0.0.0.0
NEURO_PORT=8080
NEURO_STORAGE_PATH=/data/neuro
NEURO_EMBEDDING_MODEL=minilm
NEURO_LOG_LEVEL=info
```

### Storage Options

- **Memory Storage**: Fast, ephemeral (default)
- **File Storage**: Persistent, JSON-based

```bash
# Use file storage
neuro serve --storage ./data

# Use memory storage (default)
neuro serve
```

## 🔧 Development

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- serve

# Format code
cargo fmt

# Lint
cargo clippy

# Build documentation
cargo doc --open
```

## 📊 Performance

Benchmarks on AMD Ryzen 9 5950X (32 threads):

| Operation | Time |
|-----------|------|
| Embedding (384 dims) | ~5ms |
| Similarity search (10k docs) | ~2ms |
| Query classification | ~0.1ms |
| Document indexing | ~10ms |

## 📜 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 🙏 Acknowledgments

- [fastembed](https://github.com/Anush008/fastembed-rs) - Native embedding models
- [axum](https://github.com/tokio-rs/axum) - Web framework
- [tree-sitter](https://tree-sitter.github.io/tree-sitter/) - Code parsing
- [ndarray](https://github.com/rust-ndarray/ndarray) - N-dimensional arrays
