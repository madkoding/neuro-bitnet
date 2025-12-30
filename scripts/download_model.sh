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
# Paso 1: Compilar binarios usando cmake directamente
# -----------------------------------------------------------------------------
echo ""
echo "📥 Paso 1: Compilando binarios con CMake..."

mkdir -p build
cd build

# Compilar con soporte AVX2 y configuración Release
cmake .. -DCMAKE_BUILD_TYPE=Release \
    -DLLAMA_NATIVE=OFF \
    -DLLAMA_AVX2=ON \
    -DLLAMA_AVX512=OFF \
    -DLLAMA_FMA=ON \
    -DLLAMA_F16C=ON

cmake --build . --config Release -j$(nproc)

cd ..

echo "✅ Binarios compilados correctamente"

# Verificar que los binarios existen
if [ -f "build/bin/llama-server" ]; then
    echo "📊 llama-server encontrado"
elif [ -f "build/bin/llama-cli" ]; then
    echo "📊 llama-cli encontrado"
else
    echo "⚠️  Buscando binarios en otras ubicaciones..."
    find build -name "llama-server" -o -name "llama-cli" 2>/dev/null || true
fi

echo "📊 Contenido de build/bin/:"
ls -la build/bin/ 2>/dev/null || echo "   (directorio no existe, binarios en otra ubicación)"

# -----------------------------------------------------------------------------
# Paso 2: Descargar el modelo GGUF pre-convertido
# -----------------------------------------------------------------------------
mkdir -p models

case "$MODEL_VARIANT" in
    "falcon-7b")
        echo ""
        echo "📥 Paso 2: Descargando Falcon3-7B-Instruct-1.58bit-GGUF..."
        
        mkdir -p models/falcon-7b
        
        # Descargar GGUF pre-convertido desde HuggingFace
        python3 -c "
from huggingface_hub import hf_hub_download
import os
os.makedirs('models/falcon-7b', exist_ok=True)
print('Descargando falcon3-7b-instruct-1.58bit.gguf...')
path = hf_hub_download(
    repo_id='tiiuae/Falcon3-7B-Instruct-1.58bit-GGUF',
    filename='falcon3-7b-instruct-1.58bit.gguf',
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
        
        # Descargar GGUF pre-convertido desde HuggingFace
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
# Paso 3: Limpieza
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
