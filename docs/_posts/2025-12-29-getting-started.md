---
title: Getting Started with neuro-bitnet
date: 2025-12-29 00:00:00 -0300
categories: [Documentation, Setup]
tags: [installation, quickstart, rust]
pin: false
math: false
mermaid: false
---

# Getting Started with neuro-bitnet

This guide will help you get started with neuro-bitnet, a high-performance RAG server written in Rust.

## Installation

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
```

### Build from Source

```bash
git clone https://github.com/madkoding/neuro-bitnet.git
cd neuro-bitnet
cargo build --release
```

## Quick Start

### 1. Setup BitNet (for local inference)

```bash
# Run the setup script
./scripts/setup_bitnet.sh
```

### 2. Download a Model

```bash
neuro model download 2b
```

### 3. Ask Questions

```bash
neuro ask "What is the capital of France?"
```

## CLI Commands

### Server Commands

```bash
# Start the HTTP server
neuro serve --port 8080

# Start with persistent storage
neuro serve --port 8080 --storage ./data
```

### Indexing

```bash
# Index a directory
neuro index ./src --recursive --include "*.rs"

# Show storage statistics
neuro stats --storage ./data
```

### Queries

```bash
# Execute a query
neuro query "What is Rust?" --storage ./data

# Classify a query
neuro classify "Calculate 2 + 2"
```

### Inference

```bash
# Ask a question (local inference)
neuro ask "What is BitNet?"

# With streaming output
neuro ask "Explain quantum computing" --stream

# With web search context
neuro ask "Latest news about Rust" --web
```

### Model Management

```bash
# List available models
neuro model list

# Download a model
neuro model download 2b

# Remove a model
neuro model remove 2b

# Show model info
neuro model info
```

## Architecture

neuro-bitnet is organized as a Rust workspace with multiple crates:

| Crate | Description |
|-------|-------------|
| `neuro-cli` | Command-line interface |
| `neuro-core` | Shared types and utilities |
| `neuro-embeddings` | fastembed-based embeddings |
| `neuro-storage` | Document storage |
| `neuro-classifier` | Query classification |
| `neuro-search` | Web search integration |
| `neuro-server` | HTTP API server |
| `neuro-inference` | BitNet inference |

## Next Steps

- [Local Inference Guide](/posts/local-inference-guide/) - Setup BitNet for local inference
- [API Reference](/posts/api-reference/) - HTTP API documentation
- [Benchmark Results](/posts/bitnet-benchmark-results/) - Performance analysis
