#!/bin/bash
# =============================================================================
# Script de descarga de modelos para neuro-bitnet
# Uso: ./download_model.sh <MODEL_VARIANT>
# Variantes: falcon-10b, bitnet-2b
# =============================================================================

set -e

MODEL_VARIANT="${1:-falcon-10b}"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           neuro-bitnet - Descarga de Modelo                    ║"
echo "║           Variante: $MODEL_VARIANT                             ║"
echo "╚════════════════════════════════════════════════════════════════╝"

case "$MODEL_VARIANT" in
    "falcon-10b")
        echo "📥 Descargando Falcon3-10B-Instruct-1.58bit..."
        echo "   Esto puede tomar 15-30 minutos..."
        
        # Descargar y convertir usando setup_env.py
        python3 setup_env.py \
            --hf-repo tiiuae/Falcon3-10B-Instruct-1.58bit \
            -q i2_s
        
        # Renombrar directorio para consistencia
        if [ -d "models/Falcon3-10B-Instruct-1.58bit" ]; then
            mv models/Falcon3-10B-Instruct-1.58bit models/falcon-10b
        fi
        
        echo "✅ Falcon3-10B-Instruct descargado y convertido"
        ;;
        
    "bitnet-2b")
        echo "📥 Descargando BitNet-b1.58-2B-4T (pre-convertido GGUF)..."
        
        mkdir -p models/bitnet-2b
        
        # Este modelo ya viene en formato GGUF, descarga directa
        huggingface-cli download microsoft/BitNet-b1.58-2B-4T-gguf \
            --local-dir models/bitnet-2b \
            --include "*.gguf"
        
        # Compilar binarios (setup_env.py sin modelo para compilar llama-server)
        echo "🔨 Compilando binarios BitNet..."
        python3 setup_env.py -q i2_s || true
        
        echo "✅ BitNet-b1.58-2B-4T descargado"
        ;;
        
    *)
        echo "❌ Error: Variante de modelo no soportada: $MODEL_VARIANT"
        echo "   Variantes disponibles: falcon-10b, bitnet-2b"
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
