#!/bin/bash
# =============================================================================
# neuro-bitnet Entrypoint
# Multi-model support: Falcon3-10B / BitNet-2B
# Modelo ya incluido en la imagen, solo configura GPU e inicia llama-server
# =============================================================================

set -e

# Detectar modelo desde la variable de entorno
MODEL_VARIANT="${MODEL_VARIANT:-falcon-10b}"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              neuro-bitnet - BitNet 1.58-bit LLM                ║"
echo "║              Modelo: $MODEL_VARIANT                            ║"
echo "║                   GPU Experimental Mode                        ║"
echo "╚════════════════════════════════════════════════════════════════╝"

# -----------------------------------------------------------------------------
# Verificar GPU
# -----------------------------------------------------------------------------
echo ""
echo "🔍 Verificando GPU NVIDIA..."

if command -v nvidia-smi &> /dev/null; then
    echo "✅ GPU NVIDIA detectada:"
    nvidia-smi --query-gpu=index,name,memory.total,driver_version --format=csv,noheader
    echo ""
else
    echo "⚠️  nvidia-smi no disponible. Continuando en modo CPU..."
    export BITNET_GPU_LAYERS=0
fi

# -----------------------------------------------------------------------------
# Verificar/Localizar modelo
# -----------------------------------------------------------------------------
# El modelo ya está incluido en la imagen, solo verificamos su ubicación
MODEL_PATH="${BITNET_MODEL_PATH:-/app/models/${MODEL_VARIANT}/ggml-model-i2_s.gguf}"
MODEL_DIR=$(dirname "$MODEL_PATH")

echo "📦 Verificando modelo en: $MODEL_PATH"

# Buscar el modelo en ubicaciones posibles
if [ ! -f "$MODEL_PATH" ]; then
    echo "⚠️  Modelo no encontrado en $MODEL_PATH"
    echo "   Buscando en ubicaciones alternativas..."
    
    # Buscar cualquier archivo .gguf en /app/models
    FOUND_MODEL=$(find /app/models -name "*.gguf" -type f | head -n 1)
    
    if [ -n "$FOUND_MODEL" ]; then
        echo "✅ Modelo encontrado en: $FOUND_MODEL"
        MODEL_PATH="$FOUND_MODEL"
    else
        echo "❌ Error: No se encontró ningún modelo GGUF en /app/models"
        echo "   Contenido de /app/models:"
        ls -laR /app/models 2>/dev/null || echo "   Directorio vacío o no existe"
        exit 1
    fi
else
    echo "✅ Modelo encontrado"
fi

# Verificar que el archivo existe después de la descarga
if [ ! -f "$MODEL_PATH" ]; then
    echo "❌ Error: No se pudo encontrar el modelo en $MODEL_PATH"
    echo "   Archivos disponibles en $MODEL_DIR:"
    ls -la "$MODEL_DIR" 2>/dev/null || echo "   Directorio vacío o no existe"
    exit 1
fi

# Mostrar tamaño del modelo
MODEL_SIZE=$(du -h "$MODEL_PATH" | cut -f1)
echo "📊 Tamaño del modelo: $MODEL_SIZE"

# -----------------------------------------------------------------------------
# Configurar variables
# -----------------------------------------------------------------------------
HOST="${BITNET_HOST:-0.0.0.0}"
PORT="${BITNET_PORT:-8080}"
CTX_SIZE="${BITNET_CTX_SIZE:-4096}"
PARALLEL="${BITNET_PARALLEL:-4}"
GPU_LAYERS="${BITNET_GPU_LAYERS:-99}"
THREADS="${BITNET_THREADS:-4}"

# Parámetros de sampling para mejor calidad
TEMP="${BITNET_TEMPERATURE:-0.7}"
TOP_K="${BITNET_TOP_K:-40}"
TOP_P="${BITNET_TOP_P:-0.95}"
REPEAT_PENALTY="${BITNET_REPEAT_PENALTY:-1.1}"
MIN_P="${BITNET_MIN_P:-0.05}"

# Parámetros avanzados de sampling
PRESENCE_PENALTY="${BITNET_PRESENCE_PENALTY:-0.0}"
FREQUENCY_PENALTY="${BITNET_FREQUENCY_PENALTY:-0.0}"
REPEAT_LAST_N="${BITNET_REPEAT_LAST_N:-64}"
TFS_Z="${BITNET_TFS_Z:-1.0}"
TYPICAL_P="${BITNET_TYPICAL_P:-1.0}"
MIROSTAT="${BITNET_MIROSTAT:-0}"
MIROSTAT_TAU="${BITNET_MIROSTAT_TAU:-5.0}"
MIROSTAT_ETA="${BITNET_MIROSTAT_ETA:-0.1}"

# Parámetros de rendimiento
BATCH_SIZE="${BITNET_BATCH_SIZE:-2048}"
UBATCH_SIZE="${BITNET_UBATCH_SIZE:-512}"
THREADS_BATCH="${BITNET_THREADS_BATCH:-$THREADS}"
DEFRAG_THOLD="${BITNET_DEFRAG_THOLD:-0.1}"
CACHE_REUSE="${BITNET_CACHE_REUSE:-256}"

echo ""
echo "⚙️  Configuración del servidor:"
echo "   Host:           $HOST"
echo "   Puerto:         $PORT"
echo "   Contexto:       $CTX_SIZE tokens"
echo "   Slots paralelos: $PARALLEL"
echo "   GPU Layers:     $GPU_LAYERS"
echo "   CPU Threads:    $THREADS"
echo ""
echo "🎛️  Parámetros de sampling:"
echo "   Temperature:    $TEMP"
echo "   Top-K:          $TOP_K"
echo "   Top-P:          $TOP_P"
echo "   Min-P:          $MIN_P"
echo "   Repeat Penalty: $REPEAT_PENALTY (last $REPEAT_LAST_N tokens)"
if [ "$MIROSTAT" != "0" ]; then
    echo "   Mirostat:       $MIROSTAT (tau=$MIROSTAT_TAU, eta=$MIROSTAT_ETA)"
fi
echo ""
echo "⚡ Parámetros de rendimiento:"
echo "   Batch Size:     $BATCH_SIZE"
echo "   uBatch Size:    $UBATCH_SIZE"
echo "   Defrag Thold:   $DEFRAG_THOLD"
echo "   Cache Reuse:    $CACHE_REUSE"
echo ""

# -----------------------------------------------------------------------------
# Buscar el binario llama-server
# -----------------------------------------------------------------------------
LLAMA_SERVER=""

# Posibles ubicaciones del binario
POSSIBLE_PATHS=(
    "/app/build/bin/llama-server"
    "/app/build/bin/llama-cli"
    "/app/build/bin/main"
    "/app/build/llama-server"
)

for path in "${POSSIBLE_PATHS[@]}"; do
    if [ -x "$path" ]; then
        LLAMA_SERVER="$path"
        break
    fi
done

if [ -z "$LLAMA_SERVER" ]; then
    echo "❌ Error: No se encontró el binario llama-server"
    echo "   Buscando en /app/build:"
    find /app/build -type f -executable -name "llama*" 2>/dev/null || echo "   No se encontraron binarios"
    exit 1
fi

echo "🚀 Usando binario: $LLAMA_SERVER"

# -----------------------------------------------------------------------------
# Iniciar servidor
# -----------------------------------------------------------------------------
echo ""
echo "🌐 Iniciando servidor BitNet en $HOST:$PORT..."
echo "   API OpenAI compatible disponible en:"
echo "   - POST /v1/chat/completions"
echo "   - POST /v1/completions"
echo "   - GET  /health"
echo ""
echo "   💡 Para embeddings, usa sentence-transformers localmente"
echo "      o ejecuta una segunda instancia con --embeddings"
echo ""

# Ejecutar llama-server con configuración optimizada
exec "$LLAMA_SERVER" \
    --model "$MODEL_PATH" \
    --host "$HOST" \
    --port "$PORT" \
    --ctx-size "$CTX_SIZE" \
    --parallel "$PARALLEL" \
    --n-gpu-layers "$GPU_LAYERS" \
    --threads "$THREADS" \
    --threads-batch "$THREADS_BATCH" \
    --batch-size "$BATCH_SIZE" \
    --ubatch-size "$UBATCH_SIZE" \
    --cont-batching \
    --flash-attn \
    --cache-type-k q8_0 \
    --cache-type-v q8_0 \
    --defrag-thold "$DEFRAG_THOLD" \
    --cache-reuse "$CACHE_REUSE" \
    --temp "$TEMP" \
    --top-k "$TOP_K" \
    --top-p "$TOP_P" \
    --min-p "$MIN_P" \
    --repeat-penalty "$REPEAT_PENALTY" \
    --repeat-last-n "$REPEAT_LAST_N" \
    --presence-penalty "$PRESENCE_PENALTY" \
    --frequency-penalty "$FREQUENCY_PENALTY" \
    --tfs "$TFS_Z" \
    --typical "$TYPICAL_P" \
    --mirostat "$MIROSTAT" \
    --mirostat-ent "$MIROSTAT_TAU" \
    --mirostat-lr "$MIROSTAT_ETA" \
    --slots \
    --metrics
