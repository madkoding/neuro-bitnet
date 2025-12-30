---
title: Guía de Inicio
icon: fas fa-rocket
order: 2
---

# Guía de Inicio

## Requisitos

- Python 3.10+
- Docker (opcional, recomendado)
- GPU NVIDIA con CUDA 12.x (para inferencia)

## Instalación

### Con Docker (Recomendado)

```bash
# Clonar repositorio
git clone https://github.com/madkoding/neuro-bitnet.git
cd neuro-bitnet/docker

# Iniciar con modelo BitNet
docker compose up -d

# O con modelo Falcon
BITNET_MODEL=falcon-7b docker compose up -d
```

### Con Python

```bash
# Clonar repositorio
git clone https://github.com/madkoding/neuro-bitnet.git
cd neuro-bitnet

# Crear entorno virtual
python -m venv venv
source venv/bin/activate  # Linux/Mac
# venv\Scripts\activate   # Windows

# Instalar dependencias
pip install -r requirements.txt

# Iniciar servidor
python -m src.server.rag_server
```

## Configuración

### Variables de Entorno

| Variable | Descripción | Default |
|----------|-------------|---------|
| `RAG_SERVER_PORT` | Puerto del servidor | `8080` |
| `RAG_LLM_URL` | URL del LLM backend | `http://localhost:11435` |
| `RAG_EMBEDDING_MODEL` | Modelo de embeddings | `minilm` |
| `RAG_DATA_DIR` | Directorio de datos | `~/.neuro-bitnet/rag` |

### Presets Disponibles

```bash
# Usar preset balanceado
source presets/balanced.env
python -m src.server.rag_server

# Usar preset creativo
source presets/creative.env
python -m src.server.rag_server
```

## Uso Básico

### Hacer una consulta

```bash
# Consulta simple
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "¿Cuál es la capital de Francia?"}'

# Clasificar consulta
curl -X POST http://localhost:8080/classify \
  -H "Content-Type: application/json" \
  -d '{"query": "calcula 2 + 2"}'
```

### Con el cliente CLI

```bash
# Consulta interactiva
python -m src.cli.rag_client query "¿Qué es Python?"

# Indexar proyecto
python -m src.cli.index_project /ruta/al/proyecto
```

## Siguiente Paso

Ver [Arquitectura](/neuro-bitnet/architecture/) para entender el diseño del sistema.
