---
title: API Reference
date: 2025-12-31 00:00:00 -0300
categories: [Documentation, API]
tags: [api, http, rest, server]
pin: false
math: false
mermaid: false
---

# API Reference

neuro-bitnet provides a RESTful HTTP API for document storage, search, and intelligent query processing.

## Starting the Server

```bash
# Basic
neuro serve --port 8080

# With persistent storage
neuro serve --port 8080 --storage ./data

# With custom embedding model
neuro serve --port 8080 --model bge-small
```

## Endpoints

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
