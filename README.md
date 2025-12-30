# neuro-bitnet

[![Docker Hub](https://img.shields.io/docker/v/madkoding/neuro-bitnet?label=Docker%20Hub&logo=docker)](https://hub.docker.com/r/madkoding/neuro-bitnet)
[![Docker Pulls](https://img.shields.io/docker/pulls/madkoding/neuro-bitnet?logo=docker)](https://hub.docker.com/r/madkoding/neuro-bitnet)
[![Tests](https://img.shields.io/github/actions/workflow/status/madkoding/neuro-bitnet/tests.yml?label=Tests&logo=github)](https://github.com/madkoding/neuro-bitnet/actions)
[![Documentation](https://img.shields.io/badge/docs-GitHub%20Pages-blue)](https://madkoding.github.io/neuro-bitnet/)

RAG Server inteligente con clasificación automática de consultas para modelos LLM cuantizados.

## ✨ Características

- 🧠 **Clasificación Inteligente**: Detecta automáticamente el tipo de consulta
- 🔍 **RAG Selectivo**: Solo usa RAG cuando mejora la precisión (+33% en factuales)
- 📊 **Múltiples Embeddings**: Soporte para MiniLM y MPNet
- 🐳 **Docker Ready**: Imágenes optimizadas para GPU NVIDIA
- 🧪 **Bien Testeado**: Suite completa de tests unitarios e integración

## 🚀 Inicio Rápido

### Con Docker (Recomendado)

```bash
cd docker
docker compose up -d

# Verificar estado
curl http://localhost:11435/health
```

### Con Python

```bash
# Instalar dependencias
pip install -r requirements.txt

# Iniciar servidor
python -m src.server.rag_server
```

## 📊 Uso

### Hacer una consulta

```bash
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "¿Cuál es la capital de Francia?"}'
```

### Clasificar consulta

```bash
curl -X POST http://localhost:8080/classify \
  -H "Content-Type: application/json" \
  -d '{"query": "calcula 2 + 2"}'
```

### Indexar proyecto

```bash
python -m src.cli.index_project /ruta/al/proyecto
```

## 📁 Estructura del Proyecto

```
neuro-bitnet/
├── src/
│   ├── rag/          # Módulo principal RAG
│   │   ├── classifier.py    # Clasificación de consultas
│   │   ├── embeddings.py    # Gestión de embeddings
│   │   ├── storage/         # Backends de almacenamiento
│   │   └── indexer/         # Analizadores de código
│   ├── server/       # Servidor HTTP
│   └── cli/          # Herramientas CLI
├── docker/           # Configuración Docker
├── tests/            # Tests unitarios e integración
└── docs/             # Documentación (Jekyll/Chirpy)
```

## 📈 Benchmarks

| Categoría | Sin RAG | Con RAG | Mejora |
|-----------|---------|---------|--------|
| Matemáticas | 100% | 100% | = |
| Código | 100% | 100% | = |
| Razonamiento | 100% | 100% | = |
| **Factual** | **66.7%** | **100%** | **+33%** |

Ver [análisis completo](https://madkoding.github.io/neuro-bitnet/benchmarks/).

## 🧪 Tests

```bash
# Ejecutar todos los tests
pytest

# Solo tests unitarios
pytest tests/unit/

# Con cobertura
pytest --cov=src --cov-report=html
```

## 📚 Documentación

Documentación completa disponible en [GitHub Pages](https://madkoding.github.io/neuro-bitnet/):

- [Guía de Inicio](https://madkoding.github.io/neuro-bitnet/getting-started/)
- [Arquitectura](https://madkoding.github.io/neuro-bitnet/architecture/)
- [API Reference](https://madkoding.github.io/neuro-bitnet/api/)
- [Benchmarks](https://madkoding.github.io/neuro-bitnet/benchmarks/)

## 🛠️ Configuración

| Variable | Descripción | Default |
|----------|-------------|---------|
| `RAG_SERVER_PORT` | Puerto del servidor | `8080` |
| `RAG_LLM_URL` | URL del LLM backend | `http://localhost:11435` |
| `RAG_EMBEDDING_MODEL` | Modelo de embeddings | `minilm` |

## 📄 Licencia

MIT License - ver [LICENSE](LICENSE) para detalles.
