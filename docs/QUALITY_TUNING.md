# 🎯 Guía de Optimización de Calidad para BitNet

## Parámetros de Sampling

Los parámetros más importantes para ajustar la calidad:

### Temperature (`BITNET_TEMPERATURE`)

| Valor | Uso Recomendado |
|-------|-----------------|
| 0.1-0.3 | Código, respuestas factuales, matemáticas |
| 0.5-0.7 | Balance general, Q&A |
| 0.8-1.0 | Texto creativo, brainstorming |
| >1.0 | Muy creativo/caótico (no recomendado) |

### Top-K (`BITNET_TOP_K`)

- **20-30**: Respuestas muy enfocadas
- **40** (default): Buen balance
- **50-100**: Más variedad

### Top-P (`BITNET_TOP_P`)

- **0.9**: Más conservador
- **0.95** (default): Buen balance
- **0.99**: Más diverso

### Repeat Penalty (`BITNET_REPEAT_PENALTY`)

- **1.0**: Sin penalización (puede repetirse)
- **1.1** (default): Ligera penalización
- **1.2-1.3**: Fuerte penalización (evita repeticiones)

---

## System Prompts Optimizados

### Para Código (Alta Precisión)

```json
{
  "role": "system",
  "content": "You are an expert programmer. Follow these rules strictly:\n1. Write clean, working code only\n2. Use proper syntax and indentation\n3. Add brief comments for complex logic\n4. No explanations unless asked\n5. Use modern best practices"
}
```

**Parámetros recomendados:**
```bash
BITNET_TEMPERATURE=0.3
BITNET_TOP_K=30
BITNET_TOP_P=0.9
```

### Para Chat/Asistente General

```json
{
  "role": "system", 
  "content": "You are a helpful AI assistant. Be concise and accurate. If you don't know something, say so. Answer in the same language as the question."
}
```

**Parámetros recomendados:**
```bash
BITNET_TEMPERATURE=0.7
BITNET_TOP_K=40
BITNET_TOP_P=0.95
```

### Para Tools/Function Calling

```json
{
  "role": "system",
  "content": "You are an AI with tools. When a tool is needed, respond ONLY with JSON:\n{\"tool\": \"name\", \"arguments\": {...}}\n\nAvailable tools:\n- get_weather(location, unit)\n- calculate(expression)\n- search(query)\n\nFor general knowledge, answer directly WITHOUT using tools."
}
```

**Parámetros recomendados:**
```bash
BITNET_TEMPERATURE=0.3
BITNET_TOP_K=20
BITNET_TOP_P=0.9
BITNET_REPEAT_PENALTY=1.0
```

### Para Español

```json
{
  "role": "system",
  "content": "Eres un asistente de IA experto. Responde siempre en español, de forma clara y concisa. Si no sabes algo, dilo honestamente."
}
```

### Para RAG (Retrieval Augmented Generation)

```json
{
  "role": "system",
  "content": "You are a helpful assistant. Answer questions using ONLY the provided context. If the answer is not in the context, say 'I don't have that information in the provided documents.'\n\nContext:\n{context}"
}
```

**Parámetros recomendados:**
```bash
BITNET_TEMPERATURE=0.3
BITNET_TOP_P=0.9
```

---

## Ejemplos de Requests con Parámetros

### Request de Alta Calidad para Código

```bash
curl http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "bitnet",
    "messages": [
      {"role": "system", "content": "You are an expert Python programmer. Write clean, efficient code."},
      {"role": "user", "content": "Write a function to find prime numbers up to n using Sieve of Eratosthenes."}
    ],
    "temperature": 0.3,
    "top_p": 0.9,
    "max_tokens": 500,
    "repeat_penalty": 1.1
  }'
```

### Request para Respuestas Cortas y Precisas

```bash
curl http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "bitnet",
    "messages": [
      {"role": "system", "content": "Answer in 1-2 sentences maximum. Be direct."},
      {"role": "user", "content": "What is machine learning?"}
    ],
    "temperature": 0.5,
    "max_tokens": 100
  }'
```

### Request Creativo

```bash
curl http://localhost:11435/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "bitnet",
    "messages": [
      {"role": "system", "content": "You are a creative writer. Be imaginative and engaging."},
      {"role": "user", "content": "Write a short story about a robot learning to paint."}
    ],
    "temperature": 0.9,
    "top_p": 0.95,
    "max_tokens": 500
  }'
```

---

## Configuraciones Presets

### 🎯 Preset: Preciso (código, matemáticas)

```bash
# En .env
BITNET_TEMPERATURE=0.3
BITNET_TOP_K=30
BITNET_TOP_P=0.9
BITNET_REPEAT_PENALTY=1.1
BITNET_MIN_P=0.05
```

### 💬 Preset: Balanceado (chat general)

```bash
# En .env
BITNET_TEMPERATURE=0.7
BITNET_TOP_K=40
BITNET_TOP_P=0.95
BITNET_REPEAT_PENALTY=1.1
BITNET_MIN_P=0.05
```

### ✨ Preset: Creativo (escritura, brainstorming)

```bash
# En .env
BITNET_TEMPERATURE=0.9
BITNET_TOP_K=50
BITNET_TOP_P=0.98
BITNET_REPEAT_PENALTY=1.15
BITNET_MIN_P=0.02
```

---

## Tips Adicionales

1. **Siempre usa system prompt**: Define claramente el rol y las expectativas
2. **Sé específico**: "Write Python code" > "Write code"
3. **Limita tokens**: Usa `max_tokens` apropiado para evitar respuestas muy largas
4. **Itera**: Prueba diferentes valores y ajusta según resultados
5. **Context matters**: Proporciona contexto relevante en el prompt

## Limitaciones de BitNet-2B

- Modelo pequeño (2B parámetros) - no esperes calidad GPT-4
- Entrenado principalmente en inglés
- Contexto máximo 4096 tokens
- No tiene function calling nativo
- Puede tener dificultades con razonamiento complejo
