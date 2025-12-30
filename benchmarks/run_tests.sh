#!/bin/bash
#
# BitNet Test Runner
# ==================
# Script para ejecutar todas las pruebas de neuro-bitnet
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
URL="${BITNET_URL:-http://localhost:11435}"

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}"
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              🧪 BitNet Test Suite Runner                         ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Verificar servidor
echo -e "${YELLOW}🔗 Verificando servidor en $URL...${NC}"
if ! curl -s "$URL/health" > /dev/null 2>&1; then
    echo -e "${RED}❌ El servidor no está disponible en $URL${NC}"
    echo "   Inicia el servidor con: docker-compose up -d"
    exit 1
fi
echo -e "${GREEN}✅ Servidor disponible${NC}"

# Verificar Python
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python3 no está instalado${NC}"
    exit 1
fi

# Instalar dependencias si es necesario
echo -e "\n${YELLOW}📦 Verificando dependencias...${NC}"
pip3 install -q requests 2>/dev/null || pip install -q requests

# Menú
echo -e "\n${BLUE}Selecciona el tipo de prueba:${NC}"
echo "  1) Benchmark completo (funcionalidad)"
echo "  2) Stress test (rendimiento)"
echo "  3) Ambos"
echo "  4) Quick test (solo chat básico)"
echo ""

read -p "Opción [1-4]: " option

case $option in
    1)
        echo -e "\n${BLUE}🚀 Ejecutando Benchmark...${NC}"
        python3 "$SCRIPT_DIR/benchmark.py" --url "$URL"
        ;;
    2)
        echo -e "\n${BLUE}🚀 Ejecutando Stress Test...${NC}"
        python3 "$SCRIPT_DIR/stress_test.py" --url "$URL" --requests 5 --duration 20
        ;;
    3)
        echo -e "\n${BLUE}🚀 Ejecutando Benchmark...${NC}"
        python3 "$SCRIPT_DIR/benchmark.py" --url "$URL"
        
        echo -e "\n${BLUE}🚀 Ejecutando Stress Test...${NC}"
        python3 "$SCRIPT_DIR/stress_test.py" --url "$URL" --requests 5 --duration 20
        ;;
    4)
        echo -e "\n${BLUE}🚀 Quick Test...${NC}"
        response=$(curl -s "$URL/v1/chat/completions" \
            -H "Content-Type: application/json" \
            -d '{
                "model": "bitnet",
                "messages": [{"role": "user", "content": "Hello! Say hi in one word."}],
                "max_tokens": 20
            }')
        
        content=$(echo "$response" | python3 -c "import sys,json; print(json.load(sys.stdin)['choices'][0]['message']['content'])" 2>/dev/null || echo "Error parsing response")
        echo -e "${GREEN}Response: $content${NC}"
        ;;
    *)
        echo -e "${RED}Opción inválida${NC}"
        exit 1
        ;;
esac

echo -e "\n${GREEN}✅ Tests completados${NC}"
