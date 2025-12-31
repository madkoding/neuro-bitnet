#!/usr/bin/env bash
# Benchmark de inferencia BitNet para neuro-bitnet
# Uso: ./benchmark_bitnet.sh

set -e

NEURO_BIN="${NEURO_BIN:-./target/release/neuro}"
RESULTS_DIR="benchmarks/results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/bitnet_benchmark_$TIMESTAMP.md"

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Verificar binario
if [ ! -f "$NEURO_BIN" ]; then
    echo -e "${YELLOW}Compilando neuro-cli...${NC}"
    cargo build --release -p neuro-cli
fi

# Crear directorio de resultados
mkdir -p "$RESULTS_DIR"

echo "═══════════════════════════════════════════════════════════════"
echo "  BENCHMARK DE INFERENCIA BitNet - neuro-bitnet"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Binario: $NEURO_BIN"
echo "Resultados: $RESULTS_FILE"
echo ""

# Verificar modelo
MODEL_INFO=$($NEURO_BIN model list 2>&1 || echo "error")
if echo "$MODEL_INFO" | grep -q "not found\|No models"; then
    echo -e "${YELLOW}Descargando modelo BitNet 2B...${NC}"
    $NEURO_BIN model download 2b --yes
fi

# Preguntas de prueba organizadas por categoría
declare -a GREETINGS=(
    "Hello, how are you today?"
    "What is your name?"
    "Good morning!"
)

declare -a FACTUAL=(
    "What is the capital of France?"
    "What is 2+2?"
    "How many planets are in the solar system?"
)

declare -a TECHNICAL=(
    "What is a binary search algorithm?"
    "Explain what TCP/IP is"
    "What is the difference between RAM and ROM?"
)

declare -a CREATIVE=(
    "Write a haiku about coding"
    "Tell me a short joke"
)

declare -a REASONING=(
    "If all cats are animals, and some animals are pets, can we conclude all cats are pets?"
    "What comes next: 2, 4, 8, 16, ?"
)

# Inicializar contadores
TOTAL_TESTS=0
PASSED=0
TOTAL_TIME_MS=0

# Función para ejecutar un test
run_test() {
    local category="$1"
    local question="$2"
    local test_num="$3"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo -e "\n${CYAN}Test $test_num [$category]:${NC} $question"
    
    # Medir tiempo
    START_TIME=$(date +%s%3N)
    
    # Ejecutar pregunta
    RESPONSE=$($NEURO_BIN ask "$question" --max-tokens 100 2>&1)
    EXIT_CODE=$?
    
    END_TIME=$(date +%s%3N)
    ELAPSED=$((END_TIME - START_TIME))
    TOTAL_TIME_MS=$((TOTAL_TIME_MS + ELAPSED))
    
    # Extraer solo la respuesta (después de la línea de separación)
    ANSWER=$(echo "$RESPONSE" | awk '/^═+$/{found=1; next} found && !/^═+$/' | head -20)
    
    if [ $EXIT_CODE -eq 0 ] && [ -n "$ANSWER" ]; then
        echo -e "${GREEN}✓${NC} Completado en ${ELAPSED}ms"
        PASSED=$((PASSED + 1))
        STATUS="✅"
    else
        echo -e "${RED}✗${NC} Error o respuesta vacía"
        STATUS="❌"
    fi
    
    # Guardar en resultados
    echo "### Test $test_num: $STATUS" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
    echo "**Category:** $category" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
    echo "**Query:** $question" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
    echo "**Time:** ${ELAPSED}ms" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
    echo "**Response:**" >> "$RESULTS_FILE"
    echo "\`\`\`" >> "$RESULTS_FILE"
    echo "$ANSWER" | head -10 >> "$RESULTS_FILE"
    echo "\`\`\`" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
    echo "---" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
}

# Escribir encabezado del informe
cat > "$RESULTS_FILE" << 'EOF'
# BitNet Benchmark Report - neuro-bitnet

## Test Configuration

| Parameter | Value |
|-----------|-------|
| **Model** | BitNet b1.58 2B-4T |
| **Max Tokens** | 100 |
| **Backend** | Subprocess (llama-cli) |

EOF

echo "**Date:** $(date '+%Y-%m-%d %H:%M:%S')" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"
echo "---" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"
echo "## Test Results" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

TEST_NUM=0

# Ejecutar tests por categoría
echo -e "\n${YELLOW}=== Greetings ===${NC}"
for q in "${GREETINGS[@]}"; do
    TEST_NUM=$((TEST_NUM + 1))
    run_test "Greeting" "$q" "$TEST_NUM"
done

echo -e "\n${YELLOW}=== Factual Knowledge ===${NC}"
for q in "${FACTUAL[@]}"; do
    TEST_NUM=$((TEST_NUM + 1))
    run_test "Factual" "$q" "$TEST_NUM"
done

echo -e "\n${YELLOW}=== Technical ===${NC}"
for q in "${TECHNICAL[@]}"; do
    TEST_NUM=$((TEST_NUM + 1))
    run_test "Technical" "$q" "$TEST_NUM"
done

echo -e "\n${YELLOW}=== Creative ===${NC}"
for q in "${CREATIVE[@]}"; do
    TEST_NUM=$((TEST_NUM + 1))
    run_test "Creative" "$q" "$TEST_NUM"
done

echo -e "\n${YELLOW}=== Reasoning ===${NC}"
for q in "${REASONING[@]}"; do
    TEST_NUM=$((TEST_NUM + 1))
    run_test "Reasoning" "$q" "$TEST_NUM"
done

# Calcular estadísticas
AVG_TIME=$((TOTAL_TIME_MS / TOTAL_TESTS))
PASS_RATE=$((PASSED * 100 / TOTAL_TESTS))

# Escribir resumen
cat >> "$RESULTS_FILE" << EOF

## Summary

| Metric | Value |
|--------|-------|
| **Total Tests** | $TOTAL_TESTS |
| **Passed** | $PASSED |
| **Failed** | $((TOTAL_TESTS - PASSED)) |
| **Pass Rate** | ${PASS_RATE}% |
| **Total Time** | ${TOTAL_TIME_MS}ms |
| **Average Time** | ${AVG_TIME}ms |

## Observations

- Model: BitNet b1.58 2B-4T (1.1GB quantized)
- Backend: Subprocess via llama-cli from bitnet.cpp
- No GPU acceleration (CPU only with TLS optimization)

EOF

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  RESUMEN"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo -e "Tests completados: ${GREEN}$PASSED${NC}/$TOTAL_TESTS (${PASS_RATE}%)"
echo -e "Tiempo total: ${CYAN}${TOTAL_TIME_MS}ms${NC}"
echo -e "Tiempo promedio: ${CYAN}${AVG_TIME}ms${NC} por test"
echo ""
echo "Resultados guardados en: $RESULTS_FILE"
