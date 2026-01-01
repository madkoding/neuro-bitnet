---
title: Referencia de API
date: 2025-12-31 00:00:00 -0300
categories: [Documentación, API]
tags: [api, http, rest, servidor, daemon, openai, español]
pin: false
math: false
mermaid: true
---

# Referencia de API

neuro-bitnet proporciona dos APIs HTTP:

1. **Servidor RAG** (`neuro serve`) - Almacenamiento de documentos, búsqueda y procesamiento de consultas
2. **Servidor Daemon** (`neuro-daemon`) - API de inferencia compatible con OpenAI

---

## API del Servidor RAG

Iniciar con:
```bash
neuro serve --port 8080 --storage ./data
```

URL Base: `http://localhost:8080`

### Health Check

```http
GET /health
```

**Respuesta:**
```json
{
  "status": "ok",
  "uptime_seconds": 3600
}
```

### Estadísticas

```http
GET /stats
```

**Respuesta:**
```json
{
  "documents": 1234,
  "storage_type": "file",
  "embedding_model": "all-MiniLM-L6-v2",
  "embedding_dimensions": 384
}
```

### Agregar Documento

```http
POST /add
Content-Type: application/json
```

**Solicitud:**
```json
{
  "content": "Rust es un lenguaje de programación de sistemas...",
  "metadata": {
    "source": "documentación",
    "language": "es"
  }
}
```

**Respuesta:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "success": true
}
```

### Buscar Documentos

```http
POST /search
Content-Type: application/json
```

**Solicitud:**
```json
{
  "query": "lenguaje de programación",
  "top_k": 5,
  "min_score": 0.5
}
```

**Respuesta:**
```json
{
  "results": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "content": "Rust es un lenguaje de programación de sistemas...",
      "score": 0.87,
      "metadata": {
        "source": "documentación"
      }
    }
  ],
  "query_time_ms": 15
}
```

### Clasificar Consulta

```http
POST /classify
Content-Type: application/json
```

**Solicitud:**
```json
{
  "query": "¿Cuánto es 2 + 2?"
}
```

**Respuesta:**
```json
{
  "category": "math",
  "confidence": 0.95,
  "reasoning": "Contiene expresión matemática"
}
```

### Consulta Inteligente

```http
POST /query
Content-Type: application/json
```

**Solicitud:**
```json
{
  "query": "¿Qué es Rust?",
  "top_k": 5,
  "use_web": false
}
```

**Respuesta:**
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

### Listar Documentos

```http
GET /documents
GET /documents?limit=100&offset=0
```

**Respuesta:**
```json
{
  "documents": [...],
  "total": 1234,
  "limit": 100,
  "offset": 0
}
```

### Eliminar Documento

```http
DELETE /documents/{id}
```

**Respuesta:**
```json
{
  "success": true,
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

## API del Servidor Daemon (Compatible con OpenAI)

Iniciar con:
```bash
neuro-daemon --foreground
```

URL Base: `http://localhost:11435`

### Health Check

```http
GET /health
```

**Respuesta:**
```json
{
  "status": "ok",
  "model": "bitnet-2b",
  "auto_translate": true,
  "uptime_seconds": 3600
}
```

### Generar Texto

```http
POST /v1/generate
Content-Type: application/json
```

**Solicitud:**
```json
{
  "prompt": "¿Cuál es la capital de Francia?",
  "max_tokens": 256,
  "temperature": 0.7,
  "stream": false
}
```

**Respuesta:**
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

#### Consulta en Español (Auto-Traducida)

```bash
curl -X POST http://localhost:11435/v1/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "¿Cuál es la capital de Francia?"}'
```

El daemon detecta automáticamente español y traduce a inglés para mejor precisión (56% → 100%).

### Completaciones de Chat

```http
POST /v1/chat/completions
Content-Type: application/json
```

**Solicitud:**
```json
{
  "model": "bitnet-2b",
  "messages": [
    {"role": "system", "content": "Eres un asistente útil."},
    {"role": "user", "content": "Explica la computación cuántica en términos simples."}
  ],
  "max_tokens": 512,
  "temperature": 0.7
}
```

**Respuesta:**
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
        "content": "La computación cuántica usa principios de mecánica cuántica..."
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

### Listar Modelos

```http
GET /v1/models
```

**Respuesta:**
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

## Herramientas del Servidor MCP

El servidor MCP (`neuro-mcp`) expone herramientas via JSON-RPC sobre stdio:

| Herramienta | Descripción | Parámetros |
|------|-------------|------------|
| `generate` | Genera texto | `prompt`, `max_tokens`, `temperature` |
| `translate` | Traduce a inglés | `text` |
| `ask` | Pregunta con contexto | `question`, `context`, `max_tokens` |
| `summarize` | Resume texto | `text`, `max_length` |

Ver [Guía de Integración MCP](/neuro-bitnet/posts/mcp-integration-guide-es/) para detalles.

---

## Categorías de Consulta

El clasificador categoriza automáticamente las consultas:

| Categoría | Descripción | Ejemplo |
|----------|-------------|---------|
| `math` | Cálculos matemáticos | "¿Cuánto es 15 × 7?" |
| `code` | Preguntas de programación | "¿Cómo implemento búsqueda binaria?" |
| `reasoning` | Lógica y análisis | "¿Por qué el cielo es azul?" |
| `tools` | Solicitudes de herramientas | "Buscar el clima" |
| `greeting` | Saludos casuales | "¡Hola!" |
| `factual` | Preguntas factuales | "¿Cuál es la capital de Francia?" |
| `conversational` | Conversación general | "Cuéntame sobre ti" |

---

## Modelos de Embeddings

Modelos disponibles via fastembed:

| ID del Modelo | Dimensiones | Tamaño | Velocidad |
|----------|------------|------|-------|
| `minilm` (por defecto) | 384 | ~90MB | Rápido |
| `bge-small` | 384 | ~133MB | Rápido |
| `bge-base` | 768 | ~436MB | Medio |
| `bge-large` | 1024 | ~1.3GB | Lento |
| `gte-small` | 384 | ~67MB | Rápido |
| `gte-base` | 768 | ~219MB | Medio |
| `e5-small` | 384 | ~133MB | Rápido |
| `e5-base` | 768 | ~436MB | Medio |

---

## Respuestas de Error

Todos los errores devuelven un formato consistente:

```json
{
  "error": "Mensaje de error",
  "code": "CÓDIGO_ERROR",
  "details": {}
}
```

Códigos de error comunes:
- `NOT_FOUND` - Recurso no encontrado
- `INVALID_REQUEST` - Solicitud malformada
- `INTERNAL_ERROR` - Error del servidor
- `EMBEDDING_ERROR` - Error al generar embeddings
- `INFERENCE_ERROR` - Error de inferencia del modelo
- `TRANSLATION_ERROR` - Error de traducción

---

## Compatibilidad con SDK de OpenAI

La API del daemon es compatible con el SDK de OpenAI:

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:11435/v1",
    api_key="no-necesario"
)

response = client.chat.completions.create(
    model="bitnet-2b",
    messages=[{"role": "user", "content": "¿Qué es Rust?"}]
)
print(response.choices[0].message.content)
```

---

## Próximos Pasos

- [Guía del Servidor Daemon](/neuro-bitnet/posts/daemon-server-guide-es/) - Documentación detallada del daemon
- [Guía de Integración MCP](/neuro-bitnet/posts/mcp-integration-guide-es/) - Integración IDE
- [Primeros Pasos](/neuro-bitnet/posts/getting-started-es/) - Guía de instalación
