---
layout: default
title: Local Inference
nav_order: 3
---

# Local Inference with BitNet

🌐 **[Español](es/local-inference)** | English

neuro-bitnet provides local **CPU-only** inference using Microsoft's BitNet 1.58-bit models. These models are extremely efficient thanks to their ternary quantization.

## Features

- **CPU-Only**: Optimized specifically for modern processors
- **BitNet Models**: Only 1.58-bit models from Microsoft
- **Auto-download**: Automatic model download with SHA256 verification
- **Streaming**: Real-time responses with `--stream`
- **Integrated RAG**: Combine with semantic and web search

## Requirements

### Compile bitnet.cpp

To use local inference, you need to compile the bitnet.cpp runtime:

```bash
# Option 1: Automatic script (recommended)
./scripts/setup_bitnet.sh

# Option 2: Manual
git clone https://github.com/microsoft/BitNet.git ~/.local/share/bitnet.cpp
cd ~/.local/share/bitnet.cpp
mkdir build && cd build
cmake .. -DGGML_BITNET_X86_TLS=ON
cmake --build . --config Release -j$(nproc)
mkdir -p ~/.local/bin
cp bin/llama-cli ~/.local/bin/llama-cli-bitnet
```

### System Requirements

- **clang >= 18**: Required for 1.58-bit optimizations
- **cmake >= 3.14**

#### Ubuntu/Debian
```bash
sudo apt install clang-18 cmake build-essential
```

#### Arch Linux
```bash
sudo pacman -S clang cmake
```

#### macOS
```bash
brew install llvm cmake
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
```

## Supported Models

| Model | Size | Description | ID |
|-------|------|-------------|-----|
| BitNet b1.58 2B-4T | 1.19 GB | Main model, 2B params | `2b` |
| BitNet b1.58 Large | 0.7 GB | Base 0.7B model | `large` |
| BitNet b1.58 3B | 3.3 GB | Large 3B model | `3b` |
| Llama3 8B 1.58 | 8 GB | Llama3 8B quantized to 1.58-bit | `8b` |

## Basic Usage

### Automatic Model Download

The first time you run `neuro ask`, you'll be prompted to download the model:

```bash
neuro ask "What is BitNet?"

# Skip confirmation
neuro ask "What is BitNet?" --yes

# Use a specific model
neuro ask "What is BitNet?" --model 3b
```

### Model Management

```bash
# List available models
neuro model list

# Download a specific model
neuro model download 2b

# Remove a model
neuro model remove 2b

# View cache info
neuro model info
```

### With Streaming (real-time response)

```bash
neuro ask "Explain the theory of relativity" --stream
```

### With RAG Context

```bash
# First index some documents
neuro index ./docs --recursive

# Then ask with context
neuro ask "Summarize the documentation" --storage ./data
```

### With Web Search

```bash
neuro ask "What are the latest developments in AI?" --web
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BITNET_CLI_PATH` | Auto-detect | Path to llama-cli binary |
| `NEURO_BITNET_MODELS_DIR` | `~/.cache/neuro-bitnet/models` | Model cache directory |

### CLI Options

```bash
neuro ask "question" [OPTIONS]

Options:
  --model <NAME>        Model to use (2b, large, 3b, 8b) [default: 2b]
  --max-tokens <N>      Maximum tokens to generate [default: 512]
  --temperature <F>     Temperature for sampling [default: 0.7]
  --ctx-size <N>        Context window size [default: 4096]
  --threads <N>         Number of threads [default: auto]
  --stream              Enable streaming output
  --yes                 Auto-confirm downloads
  --web                 Enable web search context
  --storage <PATH>      Path to RAG storage
  --timing              Show timing information
  --verbose             Enable verbose output
```

## Architecture

```
┌─────────────────────────────────┐
│         neuro-cli               │
│   (Rust CLI application)        │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│      SubprocessBackend          │
│   (tokio::process::Command)     │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│        llama-cli                │
│   (from bitnet.cpp)             │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│      BitNet GGUF Model          │
│   (1.58-bit quantization)       │
└─────────────────────────────────┘
```

## Troubleshooting

### Binary not found

```bash
# Check if binary exists
which llama-cli-bitnet

# Set path manually
export BITNET_CLI_PATH="/path/to/llama-cli"
```

### Model download fails

```bash
# Remove partial download
neuro model remove 2b

# Try again with force flag
neuro model download 2b --force
```

### Slow inference

- Ensure you compiled with TLS optimization: `-DGGML_BITNET_X86_TLS=ON`
- Use fewer threads if CPU is overloaded
- Consider using a smaller model (2b instead of 8b)

---

[← Back to Home](.) · [Benchmarks →](benchmarks)
