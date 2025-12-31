---
layout: default
title: Home
nav_order: 1
---

# neuro-bitnet

[![CI](https://github.com/madkoding/neuro-bitnet/actions/workflows/ci.yml/badge.svg)](https://github.com/madkoding/neuro-bitnet/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/neuro-cli.svg)](LICENSE-MIT)

🌐 **[Español](es/)** | English

A high-performance **RAG (Retrieval Augmented Generation)** server written in Rust with **BitNet 1.58-bit** local inference support.

## ✨ Features

- 🚀 **High Performance** - Native Rust with SIMD-optimized vector operations
- 🧠 **BitNet Inference** - Local CPU-only inference with Microsoft's 1.58-bit models
- 📊 **Native Embeddings** - Built-in embedding models via fastembed
- 🔍 **Semantic Search** - Fast cosine similarity search
- 🌐 **Web Search** - Wikipedia integration for knowledge augmentation
- 📦 **Single Binary** - Static compilation, no runtime dependencies

## 🚀 Quick Start

### Installation

```bash
# From releases
curl -L https://github.com/madkoding/neuro-bitnet/releases/latest/download/neuro-linux-x86_64 -o neuro
chmod +x neuro
sudo mv neuro /usr/local/bin/

# From source
cargo install neuro-cli
```

### Setup BitNet (for local inference)

```bash
# Compile bitnet.cpp
./scripts/setup_bitnet.sh

# Download a BitNet model
neuro model download 2b

# Ask questions locally
neuro ask "What is the capital of France?"
```

## 📊 BitNet Benchmark Results

| Metric | BitNet b1.58 2B-4T |
|--------|-------------------|
| **Pass Rate** | 100% |
| **Model Size** | 1.1 GB |
| **Avg Response** | 2.8s |
| **Backend** | CPU-only |

[See full benchmark report →](benchmarks)

## 📚 Documentation

- [Local Inference Guide](local-inference) - Setup BitNet for local inference
- [Benchmarks](benchmarks) - Performance comparison and test results
- [API Reference](api) - HTTP API documentation

## 🏗️ Architecture

```
┌─────────────────┐
│   neuro-cli     │  CLI Interface
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌───────┐ ┌───────┐
│  RAG  │ │BitNet │  Inference
└───┬───┘ └───┬───┘
    │         │
    ▼         ▼
┌───────┐ ┌───────┐
│Storage│ │ GGUF  │  Models
└───────┘ └───────┘
```

## 📜 License

Licensed under MIT or Apache 2.0 at your option.

---

[GitHub](https://github.com/madkoding/neuro-bitnet) · [Releases](https://github.com/madkoding/neuro-bitnet/releases) · [Issues](https://github.com/madkoding/neuro-bitnet/issues)
