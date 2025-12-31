#!/usr/bin/env bash
# Benchmark de inferencia local para neuro-bitnet
# Uso: ./benchmark_inference.sh /path/to/model.gguf

set -e

MODEL_PATH="${1:-}"
if [ -z "$MODEL_PATH" ]; then
    echo "Uso: $0 /path/to/model.gguf"
    echo ""
    echo "Descargar modelo BitNet:"
    echo "  wget https://huggingface.co/microsoft/BitNet-b1.58-2B-4T-GGUF/resolve/main/bitnet-b1.58-2b-i2_s.gguf"
    exit 1
fi

if [ ! -f "$MODEL_PATH" ]; then
    echo "Error: Modelo no encontrado: $MODEL_PATH"
    exit 1
fi

NEURO_BIN="${NEURO_BIN:-./target/release/neuro}"
if [ ! -f "$NEURO_BIN" ]; then
    echo "Compilando neuro-cli..."
    cargo build --release -p neuro-cli
fi

echo "═══════════════════════════════════════════════════════════════"
echo "  BENCHMARK DE INFERENCIA LOCAL - neuro-bitnet"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Modelo: $MODEL_PATH"
echo "Binario: $NEURO_BIN"
echo ""

# Preguntas de prueba con diferentes categorías
declare -a QUESTIONS=(
    # Conversacionales
    "Hello, how are you today?"
    "What is your name?"
    # Técnicas
    "How do I implement a binary search in Python?"
    "What is the difference between TCP and UDP?"
    # Factuales
    "What is the capital of France?"
    "When was the first computer invented?"
    # Creativas
    "Write a haiku about programming"
    "Tell me a short joke about computers"
)

declare -a CATEGORIES=(
    "Conversational"
    "Conversational"
    "TechnicalCode"
    "TechnicalCode"
    "FactualKnowledge"
    "FactualKnowledge"
    "CreativeWriting"
    "CreativeWriting"
)

TOTAL_TESTS=${#QUESTIONS[@]}
PASSED=0
TOTAL_TIME=0

echo "Ejecutando $TOTAL_TESTS pruebas..."
echo ""

for i in "${!QUESTIONS[@]}"; do
    QUESTION="${QUESTIONS[$i]}"
    EXPECTED_CAT="${CATEGORIES[$i]}"
    
    echo "───────────────────────────────────────────────────────────────"
    echo "Test $((i+1))/$TOTAL_TESTS: $QUESTION"
    echo "Categoría esperada: $EXPECTED_CAT"
    
    # Ejecutar con timing y JSON
    OUTPUT=$($NEURO_BIN ask "$QUESTION" \
        --model-path "$MODEL_PATH" \
        --max-tokens 128 \
        --temperature 0.7 \
        --format json \
        2>/dev/null || echo '{"error": true}')
    
    if echo "$OUTPUT" | grep -q '"error"'; then
        echo "❌ Error en la ejecución"
        continue
    fi
    
    # Extraer datos del JSON
    ANSWER=$(echo "$OUTPUT" | jq -r '.answer // "N/A"' 2>/dev/null || echo "N/A")
    ACTUAL_CAT=$(echo "$OUTPUT" | jq -r '.category // "Unknown"' 2>/dev/null || echo "Unknown")
    CONFIDENCE=$(echo "$OUTPUT" | jq -r '.confidence // 0' 2>/dev/null || echo "0")
    LLM_TIME=$(echo "$OUTPUT" | jq -r '.timing.llm_ms // 0' 2>/dev/null || echo "0")
    TOTAL_TEST_TIME=$(echo "$OUTPUT" | jq -r '.timing.total_ms // 0' 2>/dev/null || echo "0")
    
    TOTAL_TIME=$((TOTAL_TIME + TOTAL_TEST_TIME))
    
    # Verificar categoría
    if [[ "$ACTUAL_CAT" == "$EXPECTED_CAT" ]]; then
        echo "✅ Categoría correcta: $ACTUAL_CAT (confianza: $CONFIDENCE)"
        PASSED=$((PASSED + 1))
    else
        echo "⚠️  Categoría: $ACTUAL_CAT (esperada: $EXPECTED_CAT)"
    fi
    
    echo "⏱  Tiempo LLM: ${LLM_TIME}ms | Total: ${TOTAL_TEST_TIME}ms"
    echo "📝 Respuesta: ${ANSWER:0:200}..."
    echo ""
done

echo "═══════════════════════════════════════════════════════════════"
echo "  RESULTADOS"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Clasificación correcta: $PASSED/$TOTAL_TESTS ($(echo "scale=1; $PASSED * 100 / $TOTAL_TESTS" | bc)%)"
echo "Tiempo total: ${TOTAL_TIME}ms"
echo "Tiempo promedio: $(echo "scale=0; $TOTAL_TIME / $TOTAL_TESTS" | bc)ms por consulta"
echo ""
echo "═══════════════════════════════════════════════════════════════"
