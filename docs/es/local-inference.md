---
layout: default
title: Inferencia Local
nav_order: 3
lang: es
---

# Inferencia Local con BitNet

🌐 Español | **[English](../local-inference)**

neuro-bitnet proporciona inferencia local **solo-CPU** usando modelos BitNet 1.58-bit de Microsoft. Estos modelos son extremadamente eficientes gracias a su cuantización ternaria.

## Características

- **Solo-CPU**: Optimizado específicamente para procesadores modernos
- **Modelos BitNet**: Solo modelos de 1.58 bits de Microsoft
- **Auto-descarga**: Descarga automática de modelos con verificación SHA256
- **Streaming**: Respuestas en tiempo real con `--stream`
- **RAG integrado**: Combina con búsqueda semántica y web

## Requisitos

### Compilar bitnet.cpp

Para usar inferencia local, necesitas compilar el runtime de bitnet.cpp:

```bash
# Opción 1: Script automático (recomendado)
./scripts/setup_bitnet.sh

# Opción 2: Manual
git clone https://github.com/microsoft/BitNet.git ~/.local/share/bitnet.cpp
cd ~/.local/share/bitnet.cpp
mkdir build && cd build
cmake .. -DGGML_BITNET_X86_TLS=ON
cmake --build . --config Release -j$(nproc)
mkdir -p ~/.local/bin
cp bin/llama-cli ~/.local/bin/llama-cli-bitnet
```

### Requisitos del Sistema

- **clang >= 18**: Necesario para las optimizaciones de 1.58 bits
- **cmake >= 3.14**

#### Ubuntu/Debian
```bash
sudo apt install clang-18 cmake build-essential
```

#### Arch Linux
```bash
sudo pacman -S clang cmake
```

#### macOS
```bash
brew install llvm cmake
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
```

## Modelos Soportados

| Modelo | Tamaño | Descripción | ID |
|--------|--------|-------------|-----|
| BitNet b1.58 2B-4T | 1.19 GB | Modelo principal, 2B params | `2b` |
| BitNet b1.58 Large | 0.7 GB | Modelo base 0.7B | `large` |
| BitNet b1.58 3B | 3.3 GB | Modelo grande 3B | `3b` |
| Llama3 8B 1.58 | 8 GB | Llama3 8B cuantizado a 1.58 bits | `8b` |

## Uso Básico

### Descarga Automática de Modelos

La primera vez que ejecutes `neuro ask`, se te preguntará si deseas descargar el modelo:

```bash
neuro ask "¿Qué es BitNet?"

# Para saltar confirmación
neuro ask "¿Qué es BitNet?" --yes

# Usar un modelo específico
neuro ask "¿Qué es BitNet?" --model 3b
```

### Gestión de Modelos

```bash
# Listar modelos disponibles
neuro model list

# Descargar un modelo específico
neuro model download 2b

# Eliminar un modelo
neuro model remove 2b

# Ver información del cache
neuro model info
```

### Con Streaming (respuesta en tiempo real)

```bash
neuro ask "Explica la teoría de la relatividad" --stream
```

### Con Contexto RAG

```bash
# Primero indexa algunos documentos
neuro index ./docs --recursive

# Luego pregunta con contexto
neuro ask "Resume la documentación" --storage ./data
```

### Con Búsqueda Web

```bash
neuro ask "¿Cuáles son los últimos avances en IA?" --web
```

## Configuración

### Variables de Entorno

| Variable | Por defecto | Descripción |
|----------|-------------|-------------|
| `BITNET_CLI_PATH` | Auto-detectar | Ruta al binario llama-cli |
| `NEURO_BITNET_MODELS_DIR` | `~/.cache/neuro-bitnet/models` | Directorio de cache de modelos |

### Opciones de CLI

```bash
neuro ask "pregunta" [OPCIONES]

Opciones:
  --model <NOMBRE>      Modelo a usar (2b, large, 3b, 8b) [default: 2b]
  --max-tokens <N>      Máximo de tokens a generar [default: 512]
  --temperature <F>     Temperatura para muestreo [default: 0.7]
  --ctx-size <N>        Tamaño de ventana de contexto [default: 4096]
  --threads <N>         Número de hilos [default: auto]
  --stream              Habilitar salida en streaming
  --yes                 Auto-confirmar descargas
  --web                 Habilitar contexto de búsqueda web
  --storage <RUTA>      Ruta al almacenamiento RAG
  --timing              Mostrar información de tiempos
  --verbose             Habilitar salida detallada
```

## Arquitectura

```
┌─────────────────────────────────┐
│         neuro-cli               │
│   (Aplicación CLI en Rust)      │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│      SubprocessBackend          │
│   (tokio::process::Command)     │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│        llama-cli                │
│   (de bitnet.cpp)               │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│      Modelo BitNet GGUF         │
│   (cuantización 1.58-bit)       │
└─────────────────────────────────┘
```

## Solución de Problemas

### Binario no encontrado

```bash
# Verificar si existe el binario
which llama-cli-bitnet

# Establecer ruta manualmente
export BITNET_CLI_PATH="/ruta/a/llama-cli"
```

### Falla la descarga del modelo

```bash
# Eliminar descarga parcial
neuro model remove 2b

# Intentar de nuevo con flag force
neuro model download 2b --force
```

### Inferencia lenta

- Asegúrate de compilar con optimización TLS: `-DGGML_BITNET_X86_TLS=ON`
- Usa menos hilos si la CPU está sobrecargada
- Considera usar un modelo más pequeño (2b en vez de 8b)

---

[← Volver al Inicio](.) · [Benchmarks →](benchmarks)
