---
title: API Reference
date: 2025-12-31 00:00:00 -0300
categories: [Documentation, API]
tags: [api, http, rest, server, daemon, openai]
pin: false
math: false
mermaid: true
---

# API Reference

neuro-bitnet provides two HTTP APIs:

1. **RAG Server** (`neuro serve`) - Document storage, search, and query processing
2. **Daemon Server** (`neuro-daemon`) - OpenAI-compatible inference API

---

## RAG Server API

Start with:
```bash
neuro serve --port 8080 --storage ./data
```

Base URL: `http://localhost:8080`

### Health Check

```http
GET /health
```

**Response:**
```json
{
  "status": "ok",
  "uptime_seconds": 3600
}
```

### Statistics

```http
GET /stats
```

**Response:**
```json
{
  "documents": 1234,
  "storage_type": "file",
  "embedding_model": "all-MiniLM-L6-v2",
  "embedding_dimensions": 384
}
```

### Add Document

```http
POST /add
Content-Type: application/json
```

**Request:**
```json
{
  "content": "Rust is a systems programming language...",
  "metadata": {
    "source": "documentation",
    "language": "en"
  }
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "success": true
}
```

### Search Documents

```http
POST /search
Content-Type: application/json
```

**Request:**
```json
{
  "query": "programming language",
  "top_k": 5,
  "min_score": 0.5
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "content": "Rust is a systems programming language...",
      "score": 0.87,
      "metadata": {
        "source": "documentation"
      }
    }
  ],
  "query_time_ms": 15
}
```

### Classify Query

```http
POST /classify
Content-Type: application/json
```

**Request:**
```json
{
  "query": "What is 2 + 2?"
}
```

**Response:**
```json
{
  "category": "math",
  "confidence": 0.95,
  "reasoning": "Contains mathematical expression"
}
```

### Intelligent Query

```http
POST /query
Content-Type: application/json
```

**Request:**
```json
{
  "query": "What is Rust?",
  "top_k": 5,
  "use_web": false
}
```

**Response:**
```json
{
  "category": "factual",
  "confidence": 0.85,
  "results": [...],
  "web_results": [],
  "timing": {
    "classification_ms": 1,
    "search_ms": 12,
    "total_ms": 13
  }
}
```

### List Documents

```http
GET /documents
GET /documents?limit=100&offset=0
```

**Response:**
```json
{
  "documents": [...],
  "total": 1234,
  "limit": 100,
  "offset": 0
}
```

### Delete Document

```http
DELETE /documents/{id}
```

**Response:**
```json
{
  "success": true,
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

## Daemon Server API (OpenAI-Compatible)

Start with:
```bash
neuro-daemon --foreground
```

Base URL: `http://localhost:11435`

### Health Check

```http
GET /health
```

**Response:**
```json
{
  "status": "ok",
  "model": "bitnet-2b",
  "auto_translate": true,
  "uptime_seconds": 3600
}
```

### Generate Text

```http
POST /v1/generate
Content-Type: application/json
```

**Request:**
```json
{
  "prompt": "What is the capital of France?",
  "max_tokens": 256,
  "temperature": 0.7,
  "stream": false
}
```

**Response:**
```json
{
  "id": "gen-123456",
  "object": "text_completion",
  "created": 1735689600,
  "model": "bitnet-2b",
  "choices": [
    {
      "text": "The capital of France is Paris.",
      "index": 0,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 8,
    "completion_tokens": 7,
    "total_tokens": 15
  }
}
```

#### Spanish Query (Auto-Translated)

```bash
curl -X POST http://localhost:11435/v1/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "¿Cuál es la capital de Francia?"}'
```

The daemon automatically detects Spanish and translates to English for better accuracy (56% → 100%).

### Chat Completions

```http
POST /v1/chat/completions
Content-Type: application/json
```

**Request:**
```json
{
  "model": "bitnet-2b",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Explain quantum computing in simple terms."}
  ],
  "max_tokens": 512,
  "temperature": 0.7
}
```

**Response:**
```json
{
  "id": "chatcmpl-123456",
  "object": "chat.completion",
  "created": 1735689600,
  "model": "bitnet-2b",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Quantum computing uses quantum mechanics principles..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 150,
    "total_tokens": 170
  }
}
```

### List Models

```http
GET /v1/models
```

**Response:**
```json
{
  "object": "list",
  "data": [
    {
      "id": "bitnet-2b",
      "object": "model",
      "created": 1735689600,
      "owned_by": "local"
    }
  ]
}
```

---

## MCP Server Tools

The MCP server (`neuro-mcp`) exposes tools via JSON-RPC over stdio:

| Tool | Description | Parameters |
|------|-------------|------------|
| `generate` | Generate text | `prompt`, `max_tokens`, `temperature` |
| `translate` | Translate to English | `text` |
| `ask` | Ask with context | `question`, `context`, `max_tokens` |
| `summarize` | Summarize text | `text`, `max_length` |

See [MCP Integration Guide](/neuro-bitnet/posts/mcp-integration-guide/) for details.

---

## Query Categories

The classifier automatically categorizes queries:

| Category | Description | Example |
|----------|-------------|---------|
| `math` | Mathematical calculations | "What is 15 × 7?" |
| `code` | Programming questions | "How do I implement binary search?" |
| `reasoning` | Logic and analysis | "Why is the sky blue?" |
| `tools` | Tool/function requests | "Search for weather" |
| `greeting` | Casual greetings | "Hello!" |
| `factual` | Factual questions | "What is the capital of France?" |
| `conversational` | General conversation | "Tell me about yourself" |

---

## Embedding Models

Available models via fastembed:

| Model ID | Dimensions | Size | Speed |
|----------|------------|------|-------|
| `minilm` (default) | 384 | ~90MB | Fast |
| `bge-small` | 384 | ~133MB | Fast |
| `bge-base` | 768 | ~436MB | Medium |
| `bge-large` | 1024 | ~1.3GB | Slow |
| `gte-small` | 384 | ~67MB | Fast |
| `gte-base` | 768 | ~219MB | Medium |
| `e5-small` | 384 | ~133MB | Fast |
| `e5-base` | 768 | ~436MB | Medium |

---

## Error Responses

All errors return a consistent format:

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "details": {}
}
```

Common error codes:
- `NOT_FOUND` - Resource not found
- `INVALID_REQUEST` - Malformed request
- `INTERNAL_ERROR` - Server error
- `EMBEDDING_ERROR` - Embedding generation failed
- `INFERENCE_ERROR` - Model inference failed
- `TRANSLATION_ERROR` - Translation failed

---

## OpenAI SDK Compatibility

The daemon API is compatible with OpenAI SDK:

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:11435/v1",
    api_key="not-needed"
)

response = client.chat.completions.create(
    model="bitnet-2b",
    messages=[{"role": "user", "content": "What is Rust?"}]
)
print(response.choices[0].message.content)
```

---

## Next Steps

- [Daemon Server Guide](/neuro-bitnet/posts/daemon-server-guide/) - Detailed daemon documentation
- [MCP Integration Guide](/neuro-bitnet/posts/mcp-integration-guide/) - IDE integration
- [Getting Started](/neuro-bitnet/posts/getting-started/) - Installation guide
