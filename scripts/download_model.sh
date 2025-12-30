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

# Primero compilamos los binarios sin modelo (solo cmake)
echo "🔨 Compilando binarios BitNet/llama.cpp..."
mkdir -p build
cd build
cmake .. -G Ninja -DCMAKE_BUILD_TYPE=Release -DGGML_AVX2=ON -DGGML_AVX512=ON
ninja
cd ..
echo "✅ Binarios compilados"

case "$MODEL_VARIANT" in
    "falcon-7b")
        echo "📥 Descargando Falcon3-7B-Instruct-1.58bit-GGUF..."
        
        mkdir -p models/falcon-7b
        
        # Descargar GGUF pre-convertido desde HuggingFace
        hf download tiiuae/Falcon3-7B-Instruct-1.58bit-GGUF \
            --local-dir models/falcon-7b \
            --include "*.gguf"
        
        echo "✅ Falcon3-7B-Instruct descargado"
        ;;
        
    "bitnet-2b")
        echo "📥 Descargando BitNet-b1.58-2B-4T-GGUF..."
        
        mkdir -p models/bitnet-2b
        
        # Descargar GGUF pre-convertido desde HuggingFace
        hf download microsoft/BitNet-b1.58-2B-4T-gguf \
            --local-dir models/bitnet-2b \
            --include "*.gguf"
        
        echo "✅ BitNet-b1.58-2B-4T descargado"
        ;;
        
    *)
        echo "❌ Error: Variante de modelo no soportada: $MODEL_VARIANT"
        echo "   Variantes disponibles: falcon-7b, bitnet-2b"
        exit 1
        ;;
esac

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
