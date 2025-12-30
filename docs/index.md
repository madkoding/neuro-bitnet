---
layout: home
icon: fas fa-home
order: 1
---

## Bienvenido a neuro-bitnet

**neuro-bitnet** es un servidor RAG (Retrieval-Augmented Generation) inteligente con clasificación automática de consultas. Diseñado para trabajar con modelos LLM cuantizados como BitNet y Falcon.

### ✨ Características Principales

- 🧠 **Clasificación Inteligente**: Detecta automáticamente el tipo de consulta (matemáticas, código, factual, etc.)
- 🔍 **RAG Selectivo**: Solo usa RAG cuando mejora la precisión (+33% en consultas factuales)
- 📊 **Embeddings Eficientes**: Soporte para MiniLM y MPNet con carga lazy
- 🐳 **Docker Ready**: Imágenes optimizadas para GPU NVIDIA
- 🧪 **Bien Testeado**: Suite completa de tests unitarios e integración

### 🚀 Inicio Rápido

```bash
# Con Docker (recomendado)
cd docker
docker compose up -d

# Con Python
pip install -r requirements.txt
python -m src.server.rag_server
```

### 📈 Resultados de Benchmark

| Categoría | Sin RAG | Con RAG | Mejora |
|-----------|---------|---------|--------|
| Matemáticas | 100% | 100% | = |
| Código | 100% | 100% | = |
| Razonamiento | 100% | 100% | = |
| **Factual** | **66.7%** | **100%** | **+33%** |

### 📚 Navegación

- [Guía de Inicio](/neuro-bitnet/getting-started/) - Instalación y configuración
- [Arquitectura](/neuro-bitnet/architecture/) - Diseño del sistema
- [API Reference](/neuro-bitnet/api/) - Documentación de endpoints
- [Benchmarks](/neuro-bitnet/benchmarks/) - Análisis de rendimiento
