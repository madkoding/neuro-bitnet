# System Prompts para BitNet-2B

Este directorio contiene los system prompts optimizados para el modelo BitNet-2B-1.58bit.

## Archivos

### `system_prompt.txt`
System prompt universal para uso general. Incluye:
- Reglas de idioma (responde en el idioma del usuario)
- Formato de respuestas (conciso para datos simples)
- Conocimiento básico de capitales
- Directrices de precisión

**Uso recomendado:** Para chat general, preguntas y respuestas, generación de código.

### `system_prompt_tools.txt`
System prompt para uso con herramientas/tools. Incluye:
- Lista de herramientas disponibles
- Cuándo usar vs no usar herramientas
- Formato JSON para llamadas a herramientas
- Conocimiento general que NO requiere herramientas

**Uso recomendado:** Cuando tu aplicación necesite function calling.

## Ejemplo de uso

### Python con requests
```python
import requests

# Cargar el system prompt
with open("prompts/system_prompt.txt") as f:
    system_prompt = f.read()

# Hacer una petición
response = requests.post(
    "http://localhost:11435/v1/chat/completions",
    json={
        "model": "bitnet",
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": "¿Cuál es la capital de Francia?"}
        ],
        "max_tokens": 50,
        "temperature": 0.3
    }
)

print(response.json()["choices"][0]["message"]["content"])
# Output: París
```

### Con Tools
```python
with open("prompts/system_prompt_tools.txt") as f:
    tools_prompt = f.read()

response = requests.post(
    "http://localhost:11435/v1/chat/completions",
    json={
        "model": "bitnet",
        "messages": [
            {"role": "system", "content": tools_prompt},
            {"role": "user", "content": "¿Qué clima hace en Tokio?"}
        ],
        "max_tokens": 100,
        "temperature": 0.3
    }
)

# Output: {"tool": "get_weather", "arguments": {"location": "Tokio"}}
```

## Por qué son necesarios

El modelo BitNet-2B es pequeño (2 mil millones de parámetros) comparado con modelos como GPT-4 (>100B) o Llama-70B. Por esto:

1. **Necesita contexto explícito:** Sin system prompt, puede confundir datos básicos
2. **Beneficia de conocimiento "anclado":** Incluir datos como capitales mejora la precisión
3. **Requiere instrucciones claras:** Especificar el formato de respuesta evita verbosidad

## Personalización

Puedes modificar estos prompts para tu caso de uso:

- **Agregar conocimiento específico:** Si tu app es de un dominio, agrega datos relevantes
- **Cambiar el idioma por defecto:** Si solo usas español, simplifícalo
- **Agregar más herramientas:** Extiende `system_prompt_tools.txt` con tus funciones

## RAG Simple (Búsqueda por Keywords)

Para añadir conocimiento dinámico sin embeddings externos, usa `scripts/rag_simple.py`:

```bash
# Pregunta única
python scripts/rag_simple.py -q "¿Cuál es la capital de Francia?"

# Modo interactivo
python scripts/rag_simple.py -i

# Con base de conocimiento personalizada
python scripts/rag_simple.py --load mi_conocimiento.json -q "..."
```

### Cómo funciona

1. Extrae keywords de tu pregunta
2. Busca documentos con keywords coincidentes
3. Inyecta el contexto relevante en el system prompt
4. El modelo responde basándose en ese contexto

### Agregar conocimiento

```python
from rag_simple import SimpleRAG

rag = SimpleRAG()
rag.add_knowledge("Elon Musk fundó SpaceX en 2002.", category="tecnología")
rag.add_knowledge("Tesla fue fundada en 2003.", category="tecnología")
rag.save("mi_conocimiento.json")
```

### Ventajas

- **Sin dependencias externas:** No necesita modelo de embeddings
- **Rápido:** Búsqueda instantánea por keywords
- **Predecible:** Los resultados son determinísticos
- **Compatible:** Funciona con el servidor BitNet normal

### Cuándo usar RAG vs System Prompt

| Característica | System Prompt | RAG Simple |
|----------------|---------------|------------|
| Conocimiento fijo | ✅ Ideal | ❌ Overkill |
| Conocimiento extenso | ❌ Limitado por contexto | ✅ Escala bien |
| Datos dinámicos | ❌ Requiere reinicio | ✅ Actualizable |
| Complejidad | Baja | Media |

## Rendimiento esperado

Con estos prompts, el benchmark muestra:
- **Chat:** ~95-100%
- **Código:** ~80-100%
- **Tools:** ~80-100%
- **Razonamiento:** ~80-100%
- **Matemáticas:** ~90-100%

La variabilidad es normal en modelos generativos pequeños.
