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

# Función para compilar usando setup_env.py (ignora errores de conversión)
compile_binaries() {
    echo "🔨 Compilando binarios BitNet usando setup_env.py..."
    
    # Usar BitNet-2B para compilar (es pequeño y rápido)
    # El flag -q i2_s es necesario para la compilación
    # Permitimos que falle la conversión pero los binarios se compilan primero
    python3 setup_env.py -q i2_s 2>&1 || {
        echo "⚠️  setup_env.py terminó con error (posiblemente en conversión)"
        echo "   Verificando si los binarios fueron compilados..."
    }
    
    # Verificar que los binarios existen
    if [ -f "build/bin/llama-server" ] || [ -f "build/bin/llama-cli" ]; then
        echo "✅ Binarios compilados correctamente"
        return 0
    else
        echo "❌ Error: No se encontraron los binarios compilados"
        ls -la build/bin/ 2>/dev/null || echo "   Directorio build/bin no existe"
        return 1
    fi
}

case "$MODEL_VARIANT" in
    "falcon-7b")
        echo ""
        echo "📥 Paso 1: Compilando binarios..."
        compile_binaries
        
        echo ""
        echo "📥 Paso 2: Descargando Falcon3-7B-Instruct-1.58bit-GGUF..."
        
        # Limpiar modelo usado para compilación
        rm -rf models/* 2>/dev/null || true
        mkdir -p models/falcon-7b
        
        # Descargar GGUF pre-convertido desde HuggingFace
        python3 -c "
from huggingface_hub import hf_hub_download
import os
os.makedirs('models/falcon-7b', exist_ok=True)
print('Descargando modelo GGUF...')
hf_hub_download(
    repo_id='tiiuae/Falcon3-7B-Instruct-1.58bit-GGUF',
    filename='falcon3-7b-instruct-1.58bit.gguf',
    local_dir='models/falcon-7b'
)
print('✅ Descarga completada')
"
        
        echo "✅ Falcon3-7B-Instruct descargado"
        ;;
        
    "bitnet-2b")
        echo ""
        echo "📥 Paso 1: Compilando binarios..."
        compile_binaries
        
        echo ""
        echo "📥 Paso 2: Descargando BitNet-b1.58-2B-4T-GGUF..."
        
        # Limpiar modelo usado para compilación
        rm -rf models/* 2>/dev/null || true
        mkdir -p models/bitnet-2b
        
        # Descargar GGUF pre-convertido desde HuggingFace
        python3 -c "
from huggingface_hub import hf_hub_download
import os
os.makedirs('models/bitnet-2b', exist_ok=True)
print('Descargando modelo GGUF...')
hf_hub_download(
    repo_id='microsoft/BitNet-b1.58-2B-4T-gguf',
    filename='ggml-model-i2_s.gguf',
    local_dir='models/bitnet-2b'
)
print('✅ Descarga completada')
"
        
        echo "✅ BitNet-b1.58-2B-4T descargado"
        ;;
        
    *)
        echo "❌ Error: Variante de modelo no soportada: $MODEL_VARIANT"
        echo "   Variantes disponibles: falcon-7b, bitnet-2b"
        exit 1
        ;;
esac

# Limpiar archivos temporales para reducir tamaño de imagen
echo ""
echo "🧹 Limpiando archivos temporales..."
find models/ -name "*.safetensors" -delete 2>/dev/null || true
find models/ -name "*.bin" -delete 2>/dev/null || true
find models/ -name "*.pt" -delete 2>/dev/null || true
find models/ -name ".cache" -type d -exec rm -rf {} + 2>/dev/null || true
rm -rf /root/.cache/huggingface 2>/dev/null || true

# Mostrar resultado
echo ""
echo "📊 Modelo descargado:"
find models/ -name "*.gguf" -exec ls -lah {} \;

echo ""
echo "📊 Binarios compilados:"
ls -la build/bin/ 2>/dev/null | head -10

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
