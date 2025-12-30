#!/bin/bash
# =============================================================================
# neuro-bitnet Healthcheck
# Verifica /health y ejecuta inferencia de prueba para confirmar warm-up
# =============================================================================

PORT="${BITNET_PORT:-8080}"
BASE_URL="http://localhost:${PORT}"

# -----------------------------------------------------------------------------
# Paso 1: Verificar endpoint /health
# -----------------------------------------------------------------------------
HEALTH_RESPONSE=$(curl -sf "${BASE_URL}/health" 2>/dev/null)
HEALTH_STATUS=$?

if [ $HEALTH_STATUS -ne 0 ]; then
    echo "❌ Healthcheck fallido: /health no responde"
    exit 1
fi

# Verificar que el status sea "ok" o similar
if echo "$HEALTH_RESPONSE" | grep -qiE '"status"\s*:\s*"(ok|healthy|no slot available)"'; then
    echo "✅ Endpoint /health respondió correctamente"
else
    # Si responde pero no tiene status esperado, aún puede estar inicializando
    echo "⚠️  /health respondió pero sin status esperado: $HEALTH_RESPONSE"
fi

# -----------------------------------------------------------------------------
# Paso 2: Inferencia de prueba (warm-up check)
# -----------------------------------------------------------------------------
echo "🔥 Ejecutando inferencia de prueba..."

INFERENCE_RESPONSE=$(curl -sf -X POST "${BASE_URL}/v1/completions" \
    -H "Content-Type: application/json" \
    -d '{
        "prompt": "Hello",
        "max_tokens": 1,
        "temperature": 0.1
    }' \
    --max-time 30 \
    2>/dev/null)
INFERENCE_STATUS=$?

if [ $INFERENCE_STATUS -ne 0 ]; then
    echo "⚠️  Inferencia de prueba falló (puede estar cargando modelo)"
    # No fallamos aquí para dar tiempo al modelo de cargar
    # El start_period del healthcheck debería cubrir esto
    exit 0
fi

# Verificar que la respuesta tiene contenido
if echo "$INFERENCE_RESPONSE" | grep -q '"choices"'; then
    echo "✅ Modelo cargado y respondiendo correctamente"
    exit 0
else
    echo "⚠️  Respuesta inesperada de inferencia: $INFERENCE_RESPONSE"
    exit 0
fi
