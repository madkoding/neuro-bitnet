#!/bin/bash
# =============================================================================
# Script de descarga de modelos para neuro-bitnet
# Uso: ./download_model.sh <MODEL_VARIANT>
# Variantes: falcon-7b, bitnet-2b
# =============================================================================

set -e

MODEL_VARIANT="${1:-falcon-7b}"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           neuro-bitnet - Descarga de Modelo                    ║"
echo "║           Variante: $MODEL_VARIANT                                    ║"
echo "╚════════════════════════════════════════════════════════════════╝"

# -----------------------------------------------------------------------------
# Paso 1: Compilar binarios usando BitNet-2B (modelo pequeño, rápido)
# -----------------------------------------------------------------------------
echo ""
echo "📥 Paso 1: Compilando binarios con BitNet-2B (modelo de compilación)..."
echo "   Esto incluye: cmake + descarga modelo + conversión"

# Usar BitNet-2B para compilar - es pequeño y oficial de Microsoft
python3 setup_env.py \
    --hf-repo microsoft/BitNet-b1.58-2B-4T \
    -q i2_s

echo "✅ Binarios compilados correctamente"

# Verificar que los binarios existen
if [ ! -f "build/bin/llama-server" ] && [ ! -f "build/bin/llama-cli" ]; then
    echo "❌ Error: No se encontraron los binarios compilados"
    ls -la build/bin/ 2>/dev/null || echo "   Directorio build/bin no existe"
    exit 1
fi

echo "📊 Binarios disponibles:"
ls -la build/bin/

# -----------------------------------------------------------------------------
# Paso 2: Descargar el modelo GGUF específico según variante
# -----------------------------------------------------------------------------
case "$MODEL_VARIANT" in
    "falcon-7b")
        echo ""
        echo "📥 Paso 2: Descargando Falcon3-7B-Instruct-1.58bit-GGUF..."
        
        # Eliminar modelo de compilación
        rm -rf models/BitNet-b1.58-2B-4T 2>/dev/null || true
        mkdir -p models/falcon-7b
        
        # Descargar GGUF pre-convertido desde HuggingFace
        python3 -c "
from huggingface_hub import hf_hub_download
import os
os.makedirs('models/falcon-7b', exist_ok=True)
print('Descargando falcon3-7b-instruct-1.58bit.gguf...')
hf_hub_download(
    repo_id='tiiuae/Falcon3-7B-Instruct-1.58bit-GGUF',
    filename='falcon3-7b-instruct-1.58bit.gguf',
    local_dir='models/falcon-7b'
)
print('Descarga completada')
"
        echo "✅ Falcon3-7B-Instruct descargado"
        ;;
        
    "bitnet-2b")
        echo ""
        echo "📥 Paso 2: Modelo BitNet-2B ya está listo (usado para compilación)"
        
        # Renombrar directorio para consistencia
        if [ -d "models/BitNet-b1.58-2B-4T" ]; then
            mv models/BitNet-b1.58-2B-4T models/bitnet-2b
        fi
        
        echo "✅ BitNet-b1.58-2B-4T listo"
        ;;
        
    *)
        echo "❌ Error: Variante de modelo no soportada: $MODEL_VARIANT"
        echo "   Variantes disponibles: falcon-7b, bitnet-2b"
        exit 1
        ;;
esac

# -----------------------------------------------------------------------------
# Paso 3: Limpieza
# -----------------------------------------------------------------------------
echo ""
echo "🧹 Limpiando archivos temporales..."
find models/ -name "*.safetensors" -delete 2>/dev/null || true
find models/ -name "*.bin" -delete 2>/dev/null || true
find models/ -name "*.pt" -delete 2>/dev/null || true
find models/ -name ".cache" -type d -exec rm -rf {} + 2>/dev/null || true
rm -rf /root/.cache/huggingface 2>/dev/null || true

# Mostrar resultado
echo ""
echo "📊 Modelo final:"
find models/ -name "*.gguf" -exec ls -lah {} \;

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
