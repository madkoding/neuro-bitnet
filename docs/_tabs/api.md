---
title: API Reference
icon: fas fa-code
order: 4
---

# API Reference

El servidor RAG expone una API REST en el puerto configurado (default: 8080).

## Endpoints

### POST /query

Realizar una consulta inteligente con clasificación automática.

**Request:**
```json
{
  "query": "¿Cuál es la capital de Francia?",
  "user_id": "default",
  "include_sources": true
}
```

**Response:**
```json
{
  "answer": "La capital de Francia es París.",
  "sources": [
    {
      "source": "rag",
      "score": 0.92,
      "content": "Francia es un país europeo con capital en París..."
    }
  ],
  "classification": {
    "category": "factual",
    "strategy": "rag_then_web",
    "confidence": 0.85
  },
  "timing": {
    "total_ms": 150,
    "classification_ms": 5,
    "search_ms": 45,
    "llm_ms": 100
  }
}
```

---

### POST /classify

Clasificar una consulta sin ejecutarla.

**Request:**
```json
{
  "query": "calcula 2 + 2"
}
```

**Response:**
```json
{
  "category": "math",
  "strategy": "llm_direct",
  "confidence": 0.95,
  "reasons": ["Patrón matemático detectado: \\d+\\s*\\+\\s*\\d+"]
}
```

---

### POST /add

Añadir un documento al índice RAG.

**Request:**
```json
{
  "content": "Python es un lenguaje de programación de alto nivel.",
  "source": "manual",
  "user_id": "default",
  "metadata": {
    "topic": "programming",
    "language": "es"
  }
}
```

**Response:**
```json
{
  "id": "a1b2c3d4e5f6",
  "status": "indexed",
  "embedding_model": "minilm"
}
```

---

### POST /search

Buscar documentos similares.

**Request:**
```json
{
  "query": "programación en Python",
  "user_id": "default",
  "top_k": 5,
  "min_score": 0.5
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "a1b2c3d4e5f6",
      "content": "Python es un lenguaje de programación de alto nivel.",
      "source": "manual",
      "score": 0.87,
      "metadata": {"topic": "programming"}
    }
  ],
  "total": 1
}
```

---

### GET /health

Verificar estado del servidor.

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "llm_available": true,
  "embedding_model": "minilm",
  "documents_count": 42
}
```

---

### GET /stats

Obtener estadísticas del servidor.

**Response:**
```json
{
  "storage_type": "file",
  "total_documents": 150,
  "embedding_model": "minilm",
  "embedding_dimensions": 384,
  "uptime_seconds": 3600,
  "queries_processed": 500
}
```

---

### GET /documents

Listar documentos indexados.

**Query Parameters:**
- `user_id` (string, default: "default")
- `limit` (int, default: 50)
- `offset` (int, default: 0)

**Response:**
```json
{
  "documents": [
    {
      "id": "a1b2c3d4e5f6",
      "content": "Python es un lenguaje...",
      "source": "manual",
      "created_at": "2025-01-01T00:00:00"
    }
  ],
  "total": 150,
  "limit": 50,
  "offset": 0
}
```

---

### DELETE /documents/{id}

Eliminar un documento.

**Response:**
```json
{
  "status": "deleted",
  "id": "a1b2c3d4e5f6"
}
```

## Códigos de Error

| Código | Significado |
|--------|-------------|
| 400 | Bad Request - Parámetros inválidos |
| 404 | Not Found - Documento no encontrado |
| 500 | Internal Error - Error del servidor |
| 503 | Service Unavailable - LLM no disponible |

## Autenticación

Por defecto no hay autenticación. Para producción, se recomienda usar un proxy inverso (nginx, traefik) con autenticación.
