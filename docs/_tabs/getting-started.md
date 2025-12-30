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

**Clonar repositorio:**

```bash
git clone https://github.com/madkoding/neuro-bitnet.git
```

```bash
cd neuro-bitnet/docker
```

**Iniciar con modelo BitNet:**

```bash
docker compose up -d
```

**O con modelo Falcon:**

```bash
BITNET_MODEL=falcon-7b docker compose up -d
```

### Con Python

**Clonar repositorio:**

```bash
git clone https://github.com/madkoding/neuro-bitnet.git
```

```bash
cd neuro-bitnet
```

**Crear entorno virtual (Linux/Mac):**

```bash
python -m venv venv && source venv/bin/activate
```

**Crear entorno virtual (Windows):**

```bash
python -m venv venv && venv\Scripts\activate
```

**Instalar dependencias:**

```bash
pip install -r requirements.txt
```

**Iniciar servidor:**

```bash
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

**Usar preset balanceado:**

```bash
source presets/balanced.env && python -m src.server.rag_server
```

**Usar preset creativo:**

```bash
source presets/creative.env && python -m src.server.rag_server
```

## Uso Básico

### Hacer una consulta

**Consulta simple:**

```bash
curl -X POST http://localhost:8080/query -H "Content-Type: application/json" -d '{"query": "¿Cuál es la capital de Francia?"}'
```

**Clasificar consulta:**

```bash
curl -X POST http://localhost:8080/classify -H "Content-Type: application/json" -d '{"query": "calcula 2 + 2"}'
```

### Con el cliente CLI

**Consulta interactiva:**

```bash
python -m src.cli.rag_client query "¿Qué es Python?"
```

**Indexar proyecto:**

```bash
python -m src.cli.index_project /ruta/al/proyecto
```

## Siguiente Paso

Ver [Arquitectura](architecture) para entender el diseño del sistema.
