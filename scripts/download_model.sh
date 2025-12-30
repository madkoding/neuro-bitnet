#!/bin/bash
# =============================================================================
# Script de descarga de modelos para neuro-bitnet
# Uso: ./download_model.sh <MODEL_VARIANT>
# Variantes: falcon-7b, bitnet-2b
# =============================================================================

MODEL_VARIANT="${1:-falcon-7b}"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           neuro-bitnet - Descarga de Modelo                    ║"
echo "║           Variante: $MODEL_VARIANT                                    ║"
echo "╚════════════════════════════════════════════════════════════════╝"

# -----------------------------------------------------------------------------
# Paso 1: Compilar binarios usando setup_env.py
# NOTA: La conversión fallará pero la compilación sí funciona
# -----------------------------------------------------------------------------
echo ""
echo "📥 Paso 1: Compilando binarios con setup_env.py..."
echo "   (La conversión puede fallar, pero los binarios se compilan primero)"

# Desactivar exit on error temporalmente
set +e

# Usar BitNet-2B para compilar - setup_env.py REQUIERE un modelo
# La compilación (cmake) toma ~220s y termina exitosamente
# La conversión posterior fallará pero los binarios ya existirán
python3 setup_env.py \
    --hf-repo microsoft/BitNet-b1.58-2B-4T \
    -q i2_s

SETUP_EXIT_CODE=$?
set -e

if [ $SETUP_EXIT_CODE -ne 0 ]; then
    echo "⚠️  setup_env.py terminó con código $SETUP_EXIT_CODE (error en conversión esperado)"
fi

# Verificar que los binarios existen
echo ""
echo "🔍 Verificando binarios compilados..."

if [ -f "build/bin/llama-server" ]; then
    echo "✅ llama-server encontrado"
    ls -la build/bin/llama-server
elif [ -f "build/bin/llama-cli" ]; then
    echo "✅ llama-cli encontrado"
    ls -la build/bin/llama-cli
else
    echo "❌ Error: No se encontraron binarios en build/bin/"
    echo "   Contenido del directorio build:"
    find build -type f -name "llama*" 2>/dev/null || echo "   No hay archivos llama*"
    exit 1
fi

# -----------------------------------------------------------------------------
# Paso 2: Descargar el modelo GGUF pre-convertido
# -----------------------------------------------------------------------------
echo ""
echo "🧹 Limpiando modelo de compilación..."
rm -rf models/* 2>/dev/null || true
mkdir -p models

case "$MODEL_VARIANT" in
    "falcon-7b")
        echo ""
        echo "📥 Paso 2: Descargando Falcon3-7B-Instruct-1.58bit-GGUF..."
        
        mkdir -p models/falcon-7b
        
        python3 -c "
from huggingface_hub import hf_hub_download
import os
os.makedirs('models/falcon-7b', exist_ok=True)
print('Descargando ggml-model-i2_s.gguf...')
path = hf_hub_download(
    repo_id='tiiuae/Falcon3-7B-Instruct-1.58bit-GGUF',
    filename='ggml-model-i2_s.gguf',
    local_dir='models/falcon-7b'
)
print(f'Descargado en: {path}')
"
        echo "✅ Falcon3-7B-Instruct descargado"
        ;;
        
    "bitnet-2b")
        echo ""
        echo "📥 Paso 2: Descargando BitNet-b1.58-2B-4T-GGUF..."
        
        mkdir -p models/bitnet-2b
        
        python3 -c "
from huggingface_hub import hf_hub_download
import os
os.makedirs('models/bitnet-2b', exist_ok=True)
print('Descargando ggml-model-i2_s.gguf...')
path = hf_hub_download(
    repo_id='microsoft/BitNet-b1.58-2B-4T-gguf',
    filename='ggml-model-i2_s.gguf',
    local_dir='models/bitnet-2b'
)
print(f'Descargado en: {path}')
"
        echo "✅ BitNet-b1.58-2B-4T descargado"
        ;;
        
    *)
        echo "❌ Error: Variante de modelo no soportada: $MODEL_VARIANT"
        echo "   Variantes disponibles: falcon-7b, bitnet-2b"
        exit 1
        ;;
esac

# -----------------------------------------------------------------------------
# Paso 3: Limpieza final
# -----------------------------------------------------------------------------
echo ""
echo "🧹 Limpiando archivos temporales..."
find models/ -name ".cache" -type d -exec rm -rf {} + 2>/dev/null || true
rm -rf /root/.cache/huggingface 2>/dev/null || true

# Mostrar resultado
echo ""
echo "📊 Modelo final:"
find models/ -name "*.gguf" -exec ls -lah {} \;

echo ""
echo "📊 Binarios:"
ls -la build/bin/ | grep -E "llama|main" || true

echo ""
echo "✅ Build completado para variante: $MODEL_VARIANT"

# Limpiar archivos temporales para reducir tamaño de imagen
echo "🧹 Limpiando archivos temporales..."
find models/ -name "*.safetensors" -delete 2>/dev/null || true
find models/ -name "*.bin" -delete 2>/dev/null || true
find models/ -name "*.pt" -delete 2>/dev/null || true
rm -rf /root/.cache/huggingface 2>/dev/null || true

# Mostrar resultado
echo ""
echo "📊 Modelo descargado:"
ls -lah models/*/ggml-model-i2_s.gguf 2>/dev/null || ls -lah models/*/*.gguf 2>/dev/null || echo "Archivos GGUF:"
find models/ -name "*.gguf" -exec ls -lah {} \;

echo ""
echo "✅ Descarga completada para variante: $MODEL_VARIANT"
