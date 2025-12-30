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

<pre><code class="language-json">{
  "query": "¿Cuál es la capital de Francia?",
  "user_id": "default",
  "include_sources": true
}</code></pre>

**Response:**

<pre><code class="language-json">{
  "answer": "La capital de Francia es París.",
  "sources": [
    {
      "source": "rag",
      "score": 0.92,
      "content": "Francia es un país europeo..."
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
}</code></pre>

---

### POST /classify

Clasificar una consulta sin ejecutarla.

**Request:**

<pre><code class="language-json">{
  "query": "calcula 2 + 2"
}</code></pre>

**Response:**

<pre><code class="language-json">{
  "category": "math",
  "strategy": "llm_direct",
  "confidence": 0.95,
  "reasons": ["Patrón matemático detectado"]
}</code></pre>

---

### POST /add

Añadir un documento al índice RAG.

**Request:**

<pre><code class="language-json">{
  "content": "Python es un lenguaje de programación.",
  "source": "manual",
  "user_id": "default",
  "metadata": {
    "topic": "programming",
    "language": "es"
  }
}</code></pre>

**Response:**

<pre><code class="language-json">{
  "id": "a1b2c3d4e5f6",
  "status": "indexed",
  "embedding_model": "minilm"
}</code></pre>

---

### POST /search

Buscar documentos similares.

**Request:**

<pre><code class="language-json">{
  "query": "programación en Python",
  "user_id": "default",
  "top_k": 5,
  "min_score": 0.5
}</code></pre>

**Response:**

<pre><code class="language-json">{
  "results": [
    {
      "id": "a1b2c3d4e5f6",
      "content": "Python es un lenguaje...",
      "source": "manual",
      "score": 0.87,
      "metadata": {
        "topic": "programming"
      }
    }
  ],
  "total": 1
}</code></pre>

---

### GET /health

Verificar estado del servidor.

**Response:**

<pre><code class="language-json">{
  "status": "healthy",
  "version": "1.0.0",
  "llm_available": true,
  "embedding_model": "minilm",
  "documents_count": 42
}</code></pre>

---

### GET /stats

Obtener estadísticas del servidor.

**Response:**

<pre><code class="language-json">{
  "storage_type": "file",
  "total_documents": 150,
  "embedding_model": "minilm",
  "embedding_dimensions": 384,
  "uptime_seconds": 3600,
  "queries_processed": 500
}</code></pre>

---

### GET /documents

Listar documentos indexados.

**Query Parameters:**

| Parámetro | Tipo | Default | Descripción |
|-----------|------|---------|-------------|
| `user_id` | string | "default" | ID del usuario |
| `limit` | int | 50 | Máximo de resultados |
| `offset` | int | 0 | Desplazamiento |

**Response:**

<pre><code class="language-json">{
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
}</code></pre>

---

### DELETE /documents/{id}

Eliminar un documento.

**Response:**

<pre><code class="language-json">{
  "status": "deleted",
  "id": "a1b2c3d4e5f6"
}</code></pre>

## Códigos de Error

| Código | Significado |
|--------|-------------|
| 400 | Bad Request - Parámetros inválidos |
| 404 | Not Found - Documento no encontrado |
| 500 | Internal Error - Error del servidor |
| 503 | Service Unavailable - LLM no disponible |

## Autenticación

Por defecto no hay autenticación. Para producción, se recomienda usar un proxy inverso (nginx, traefik) con autenticación.
