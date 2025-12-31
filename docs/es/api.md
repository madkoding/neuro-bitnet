---
layout: default
title: Referencia de API
nav_order: 4
lang: es
---

# Referencia de API

🌐 Español | **[English](../api)**

neuro-bitnet proporciona una API HTTP RESTful para almacenamiento de documentos, búsqueda y procesamiento inteligente de consultas.

## Iniciar el Servidor

```bash
# Básico
neuro serve --port 8080

# Con almacenamiento persistente
neuro serve --port 8080 --storage ./data

# Con modelo de embedding personalizado
neuro serve --port 8080 --model bge-small
```

## Endpoints

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
    "source": "documentation",
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
        "source": "documentation"
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

## Categorías de Consulta

El clasificador categoriza automáticamente las consultas:

| Categoría | Descripción | Ejemplo |
|-----------|-------------|---------|
| `math` | Cálculos matemáticos | "¿Cuánto es 15 × 7?" |
| `code` | Preguntas de programación | "¿Cómo implemento búsqueda binaria?" |
| `reasoning` | Lógica y análisis | "¿Por qué el cielo es azul?" |
| `tools` | Solicitudes de herramientas | "Buscar el clima" |
| `greeting` | Saludos casuales | "¡Hola!" |
| `factual` | Preguntas factuales | "¿Cuál es la capital de Francia?" |
| `conversational` | Conversación general | "Cuéntame sobre ti" |

## Modelos de Embedding

Modelos disponibles via fastembed:

| ID del Modelo | Dimensiones | Tamaño | Velocidad |
|---------------|-------------|--------|-----------|
| `minilm` (default) | 384 | ~90MB | Rápido |
| `bge-small` | 384 | ~133MB | Rápido |
| `bge-base` | 768 | ~436MB | Medio |
| `bge-large` | 1024 | ~1.3GB | Lento |
| `gte-small` | 384 | ~67MB | Rápido |
| `gte-base` | 768 | ~219MB | Medio |
| `e5-small` | 384 | ~133MB | Rápido |
| `e5-base` | 768 | ~436MB | Medio |

## Respuestas de Error

Todos los errores retornan un formato consistente:

```json
{
  "error": "Mensaje de error",
  "code": "ERROR_CODE",
  "details": {}
}
```

Códigos de error comunes:
- `NOT_FOUND` - Recurso no encontrado
- `INVALID_REQUEST` - Solicitud malformada
- `INTERNAL_ERROR` - Error del servidor
- `EMBEDDING_ERROR` - Fallo en generación de embedding

## Límite de Peticiones

El servidor no implementa límite de peticiones por defecto. Usa un proxy reverso (nginx, traefik) para despliegues en producción.

## CORS

CORS está habilitado por defecto permitiendo todos los orígenes. Configura apropiadamente para producción.

---

[← Volver al Inicio](.) · [Inferencia Local →](local-inference)
