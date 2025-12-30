#!/usr/bin/env python3
"""
BitNet Benchmark Suite
======================
Pruebas completas de rendimiento y funcionalidad para neuro-bitnet.

Incluye:
- Pruebas de chat completion
- Pruebas de generación de código
- Pruebas de tools/function calling
- Pruebas de razonamiento
- Medición de tiempos de respuesta
- Validación de respuestas correctas

Uso:
    python benchmark.py [--url URL] [--verbose]
"""

import json
import time
import re
import argparse
from dataclasses import dataclass
from typing import Callable, Optional, Any
from enum import Enum

try:
    import requests
except ImportError:
    print("❌ Falta el módulo 'requests'. Instálalo con: pip install requests")
    exit(1)

# =============================================================================
# Configuración
# =============================================================================

DEFAULT_URL = "http://localhost:11435"
TIMEOUT = 120  # segundos

class TestCategory(Enum):
    CHAT = "💬 Chat"
    CODE = "💻 Código"
    TOOLS = "🔧 Tools"
    REASONING = "🧠 Razonamiento"
    MATH = "🔢 Matemáticas"
    SPANISH = "🇪🇸 Español"

@dataclass
class TestResult:
    name: str
    category: TestCategory
    passed: bool
    response_time: float
    tokens_generated: int
    tokens_per_second: float
    response: str
    expected: str
    error: Optional[str] = None

# =============================================================================
# Cliente BitNet
# =============================================================================

class BitNetClient:
    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
    
    def health_check(self) -> bool:
        try:
            r = self.session.get(f"{self.base_url}/health", timeout=5)
            return r.status_code == 200 and r.json().get("status") == "ok"
        except:
            return False
    
    def chat(self, messages: list, max_tokens: int = 200, temperature: float = 0.3) -> dict:
        start = time.time()
        r = self.session.post(
            f"{self.base_url}/v1/chat/completions",
            json={
                "model": "bitnet",
                "messages": messages,
                "max_tokens": max_tokens,
                "temperature": temperature
            },
            timeout=TIMEOUT
        )
        elapsed = time.time() - start
        data = r.json()
        
        content = data.get("choices", [{}])[0].get("message", {}).get("content", "")
        usage = data.get("usage", {})
        tokens = usage.get("completion_tokens", 0)
        
        return {
            "content": content,
            "time": elapsed,
            "tokens": tokens,
            "tps": tokens / elapsed if elapsed > 0 else 0
        }
    
    def completion(self, prompt: str, max_tokens: int = 200, temperature: float = 0.3) -> dict:
        start = time.time()
        r = self.session.post(
            f"{self.base_url}/v1/completions",
            json={
                "prompt": prompt,
                "max_tokens": max_tokens,
                "temperature": temperature
            },
            timeout=TIMEOUT
        )
        elapsed = time.time() - start
        data = r.json()
        
        content = data.get("content", data.get("choices", [{}])[0].get("text", ""))
        tokens = data.get("tokens_predicted", 0)
        
        return {
            "content": content,
            "time": elapsed,
            "tokens": tokens,
            "tps": tokens / elapsed if elapsed > 0 else 0
        }

# =============================================================================
# Definición de Tests
# =============================================================================

# System prompt universal - funciona para todos los casos
UNIVERSAL_SYSTEM_PROMPT = """Eres un asistente de IA preciso y útil. Sigue estas reglas:

IDIOMA: Responde en el mismo idioma que el usuario.

FORMATO:
- Preguntas simples (capitales, datos): responde solo con el dato
- Matemáticas: muestra el resultado numérico primero
- Código: proporciona código funcional y limpio
- Explicaciones: sé claro y conciso

CONOCIMIENTO:
- París es la capital de Francia
- Madrid es la capital de España
- Londres es la capital de Reino Unido
- Tokio es la capital de Japón"""

# System prompt para tools - extiende el universal
SYSTEM_PROMPT_TOOLS = """Eres un asistente de IA preciso con acceso a herramientas.

HERRAMIENTAS DISPONIBLES:
- get_weather(location, unit): Obtener clima de una ciudad
- calculate(expression): Calcular expresiones matemáticas
- search_database(query): Buscar en base de datos
- send_email(to, subject, body): Enviar correo electrónico

CUÁNDO USAR HERRAMIENTAS:
✅ USA herramientas para: clima actual, búsquedas en BD, enviar correos, cálculos complejos
❌ NO uses herramientas para: conocimiento general, matemáticas simples, preguntas sobre conceptos

FORMATO DE RESPUESTA CON HERRAMIENTA:
Responde SOLO con JSON: {"tool": "nombre", "arguments": {"param": "valor"}}

CONOCIMIENTO GENERAL (responde directo, SIN herramientas):
- París es la capital de Francia
- Madrid es la capital de España
- Python es un lenguaje de programación"""

TESTS = [
    # =========================================================================
    # Chat Tests
    # =========================================================================
    {
        "name": "Saludo básico",
        "category": TestCategory.CHAT,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "¡Hola! ¿Cómo estás?"}
        ],
        "max_tokens": 50,
        "validator": lambda r: len(r) > 10 and any(w in r.lower() for w in ["hola", "bien", "ayudar", "asistente", "gusto"])
    },
    {
        "name": "Capital de Francia",
        "category": TestCategory.CHAT,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "¿Cuál es la capital de Francia?"}
        ],
        "max_tokens": 20,
        "validator": lambda r: "paris" in r.lower() or "parís" in r.lower()
    },
    {
        "name": "Seguir instrucciones",
        "category": TestCategory.CHAT,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Resume en una palabra: ¿Qué es H2O?"}
        ],
        "max_tokens": 10,
        "validator": lambda r: "agua" in r.lower() or "water" in r.lower()
    },
    
    # =========================================================================
    # Code Tests
    # =========================================================================
    {
        "name": "Hola Mundo Python",
        "category": TestCategory.CODE,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Escribe un programa Python que imprima 'Hola Mundo'. Solo código, sin explicación."}
        ],
        "max_tokens": 50,
        "validator": lambda r: "print" in r.lower() and ("hola" in r.lower() or "mundo" in r.lower() or "hello" in r.lower())
    },
    {
        "name": "Función factorial",
        "category": TestCategory.CODE,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Escribe una función en Python llamada 'factorial' que calcule el factorial de forma recursiva."}
        ],
        "max_tokens": 150,
        "validator": lambda r: "def factorial" in r and "return" in r and ("factorial(" in r or "factorial (" in r)
    },
    {
        "name": "Fibonacci iterativo",
        "category": TestCategory.CODE,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Escribe una función en Python para calcular el n-ésimo número de Fibonacci usando iteración (no recursión)."}
        ],
        "max_tokens": 200,
        "validator": lambda r: "def " in r and ("for" in r or "while" in r) and "return" in r
    },
    {
        "name": "Clase Python",
        "category": TestCategory.CODE,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Escribe una clase Python llamada 'Persona' con atributos nombre y edad, y un método saludar."}
        ],
        "max_tokens": 200,
        "validator": lambda r: ("class Persona" in r or "class Person" in r) and "__init__" in r and "def " in r
    },
    {
        "name": "Consulta SQL",
        "category": TestCategory.CODE,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Escribe una consulta SQL para seleccionar todos los usuarios mayores de 18 años de una tabla 'usuarios'."}
        ],
        "max_tokens": 100,
        "validator": lambda r: "select" in r.lower() and "from" in r.lower() and ("where" in r.lower() or "18" in r)
    },
    
    # =========================================================================
    # Tools Tests
    # =========================================================================
    {
        "name": "Tool: Consulta clima",
        "category": TestCategory.TOOLS,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT_TOOLS},
            {"role": "user", "content": "Consulta el clima actual en Tokio usando la herramienta get_weather."}
        ],
        "max_tokens": 150,
        "validator": lambda r: _is_valid_tool_call(r, "get_weather")
    },
    {
        "name": "Tool: Calculadora",
        "category": TestCategory.TOOLS,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT_TOOLS},
            {"role": "user", "content": "Usa la herramienta calculate para calcular: 15 * 23 + 100"}
        ],
        "max_tokens": 150,
        "validator": lambda r: _is_valid_tool_call(r, "calculate")
    },
    {
        "name": "Tool: Buscar en base de datos",
        "category": TestCategory.TOOLS,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT_TOOLS},
            {"role": "user", "content": "Usa search_database para buscar clientes llamados Juan."}
        ],
        "max_tokens": 150,
        "validator": lambda r: _is_valid_tool_call(r, "search_database")
    },
    {
        "name": "Tool: Enviar correo",
        "category": TestCategory.TOOLS,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT_TOOLS},
            {"role": "user", "content": "Usa send_email para enviar un correo a juan@ejemplo.com con asunto 'Reunión'."}
        ],
        "max_tokens": 200,
        "validator": lambda r: _is_valid_tool_call(r, "send_email")
    },
    {
        "name": "Respuesta directa (sin tool)",
        "category": TestCategory.TOOLS,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "¿Qué es Python?"}
        ],
        "max_tokens": 150,
        "validator": lambda r: "python" in r.lower() or "programación" in r.lower() or "lenguaje" in r.lower()
    },
    
    # =========================================================================
    # Reasoning Tests
    # =========================================================================
    {
        "name": "Razonamiento lógico simple",
        "category": TestCategory.REASONING,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Si todos los gatos son animales, y Michi es un gato, ¿qué es Michi?"}
        ],
        "max_tokens": 50,
        "validator": lambda r: "animal" in r.lower()
    },
    {
        "name": "Secuencia numérica",
        "category": TestCategory.REASONING,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "¿Qué número sigue en esta secuencia: 2, 4, 6, 8, ?"}
        ],
        "max_tokens": 50,
        "validator": lambda r: "10" in r
    },
    {
        "name": "Problema de palabras",
        "category": TestCategory.REASONING,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Juan tiene 5 manzanas. Le da 2 a María. ¿Cuántas manzanas tiene Juan ahora?"}
        ],
        "max_tokens": 50,
        "validator": lambda r: "3" in r
    },
    
    # =========================================================================
    # Math Tests
    # =========================================================================
    {
        "name": "Suma simple",
        "category": TestCategory.MATH,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "¿Cuánto es 25 + 17? Responde solo con el número."}
        ],
        "max_tokens": 20,
        "validator": lambda r: "42" in r
    },
    {
        "name": "Multiplicación",
        "category": TestCategory.MATH,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "¿Cuánto es 12 * 11? Responde solo con el número."}
        ],
        "max_tokens": 20,
        "validator": lambda r: "132" in r
    },
    {
        "name": "Porcentaje",
        "category": TestCategory.MATH,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT + "\n\nIMPORTANTE: Responde solo con el número, sin explicación."},
            {"role": "user", "content": "¿Cuánto es el 20% de 150?"}
        ],
        "max_tokens": 10,
        "validator": lambda r: "30" in r
    },
    
    # =========================================================================
    # Spanish Tests
    # =========================================================================
    {
        "name": "Capital de España",
        "category": TestCategory.SPANISH,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "¿Cuál es la capital de España?"}
        ],
        "max_tokens": 50,
        "validator": lambda r: "madrid" in r.lower()
    },
    {
        "name": "Traducción español-inglés",
        "category": TestCategory.SPANISH,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Traduce al inglés: 'Hola, ¿cómo estás?'"}
        ],
        "max_tokens": 50,
        "validator": lambda r: any(w in r.lower() for w in ["hello", "how are you", "hi"])
    },
    {
        "name": "Código con comentarios en español",
        "category": TestCategory.SPANISH,
        "messages": [
            {"role": "system", "content": UNIVERSAL_SYSTEM_PROMPT},
            {"role": "user", "content": "Escribe una función en Python que sume dos números."}
        ],
        "max_tokens": 100,
        "validator": lambda r: "def " in r and "return" in r
    },
]

# =============================================================================
# Helpers
# =============================================================================

def _is_valid_tool_call(response: str, expected_tool: str, required_args: list | None = None) -> bool:
    """Verifica si la respuesta es una llamada válida a la tool esperada."""
    try:
        # Buscar JSON anidado (puede tener {} dentro de arguments)
        start = response.find('{')
        if start == -1:
            return False
        
        # Encontrar el } correspondiente contando llaves
        depth = 0
        end = start
        for i, char in enumerate(response[start:], start):
            if char == '{':
                depth += 1
            elif char == '}':
                depth -= 1
                if depth == 0:
                    end = i + 1
                    break
        
        json_str = response[start:end]
        
        # Si el JSON está truncado (no se cerró), intentar repararlo
        if depth > 0:
            # Agregar los } faltantes
            json_str = response[start:] + ('}' * depth)
        
        # Intentar parsear JSON
        try:
            data = json.loads(json_str)
        except json.JSONDecodeError:
            # Si falla, intentar extraer tool con regex como fallback
            tool_match = re.search(r'"tool"\s*:\s*"([^"]+)"', response)
            if tool_match:
                tool_name = tool_match.group(1).replace("__", "_")
                return tool_name == expected_tool
            return False
        
        # Verificar que tiene la estructura correcta
        if "tool" not in data:
            return False
        
        # Verificar el nombre de la tool (tolerante a typos menores como doble guión bajo)
        tool_name = data["tool"].replace("__", "_")  # Normalizar doble guión bajo
        if tool_name != expected_tool:
            return False
        
        # Verificar argumentos requeridos
        # El modelo puede poner args dentro de "arguments" o directamente en el JSON raíz
        if required_args:
            # Buscar en "arguments", "args", o directamente en data
            args = data.get("arguments", data.get("args", data))
            for arg in required_args:
                if arg not in args:
                    return False
        
        return True
    except (json.JSONDecodeError, KeyError):
        # Último intento: buscar el nombre de la tool con regex
        tool_match = re.search(r'"tool"\s*:\s*"([^"]+)"', response)
        if tool_match:
            tool_name = tool_match.group(1).replace("__", "_")
            return tool_name == expected_tool
        return False

def _looks_like_tool_call(response: str) -> bool:
    """Verifica si la respuesta parece una llamada a tool."""
    try:
        start = response.find('{')
        if start == -1:
            return False
        
        depth = 0
        end = start
        for i, char in enumerate(response[start:], start):
            if char == '{':
                depth += 1
            elif char == '}':
                depth -= 1
                if depth == 0:
                    end = i + 1
                    break
        
        json_str = response[start:end]
        data = json.loads(json_str)
        return "tool" in data
    except:
        pass
    return False

# =============================================================================
# Runner
# =============================================================================

def run_tests(client: BitNetClient, verbose: bool = False) -> list[TestResult]:
    results = []
    total = len(TESTS)
    
    print("\n" + "=" * 70)
    print("🚀 BitNet Benchmark Suite")
    print("=" * 70)
    
    for i, test in enumerate(TESTS, 1):
        name = test["name"]
        category = test["category"]
        
        print(f"\n[{i}/{total}] {category.value} | {name}...")
        
        try:
            response = client.chat(
                messages=test["messages"],
                max_tokens=test.get("max_tokens", 200),
                temperature=test.get("temperature", 0.3)
            )
            
            content = response["content"]
            passed = test["validator"](content)
            
            result = TestResult(
                name=name,
                category=category,
                passed=passed,
                response_time=response["time"],
                tokens_generated=response["tokens"],
                tokens_per_second=response["tps"],
                response=content[:200] + "..." if len(content) > 200 else content,
                expected=test["validator"].__doc__ or "Custom validation"
            )
            
            status = "✅ PASS" if passed else "❌ FAIL"
            print(f"    {status} | {response['time']:.2f}s | {response['tokens']} tokens | {response['tps']:.1f} t/s")
            
            if verbose or not passed:
                print(f"    Response: {content[:100]}...")
            
        except Exception as e:
            result = TestResult(
                name=name,
                category=category,
                passed=False,
                response_time=0,
                tokens_generated=0,
                tokens_per_second=0,
                response="",
                expected="",
                error=str(e)
            )
            print(f"    ❌ ERROR: {e}")
        
        results.append(result)
    
    return results

def print_summary(results: list[TestResult]):
    """Imprime tabla resumen de resultados."""
    
    print("\n" + "=" * 90)
    print("📊 RESUMEN DE RESULTADOS")
    print("=" * 90)
    
    # Tabla detallada
    print(f"\n{'Test':<40} {'Categoría':<15} {'Estado':<8} {'Tiempo':<10} {'Tokens':<8} {'T/s':<8}")
    print("-" * 90)
    
    for r in results:
        status = "✅" if r.passed else "❌"
        print(f"{r.name:<40} {r.category.value:<15} {status:<8} {r.response_time:>6.2f}s {r.tokens_generated:>6} {r.tokens_per_second:>6.1f}")
    
    print("-" * 90)
    
    # Estadísticas por categoría
    print("\n📈 ESTADÍSTICAS POR CATEGORÍA")
    print("-" * 70)
    
    categories = {}
    for r in results:
        cat = r.category.value
        if cat not in categories:
            categories[cat] = {"passed": 0, "failed": 0, "total_time": 0, "total_tokens": 0}
        
        if r.passed:
            categories[cat]["passed"] += 1
        else:
            categories[cat]["failed"] += 1
        categories[cat]["total_time"] += r.response_time
        categories[cat]["total_tokens"] += r.tokens_generated
    
    print(f"{'Categoría':<20} {'Pasaron':<10} {'Fallaron':<10} {'% Éxito':<10} {'Tiempo Total':<15} {'Tokens':<10}")
    print("-" * 70)
    
    for cat, stats in categories.items():
        total = stats["passed"] + stats["failed"]
        pct = (stats["passed"] / total * 100) if total > 0 else 0
        print(f"{cat:<20} {stats['passed']:<10} {stats['failed']:<10} {pct:>6.1f}% {stats['total_time']:>10.2f}s {stats['total_tokens']:>10}")
    
    # Totales generales
    print("\n" + "=" * 70)
    total_passed = sum(1 for r in results if r.passed)
    total_failed = sum(1 for r in results if not r.passed)
    total_time = sum(r.response_time for r in results)
    total_tokens = sum(r.tokens_generated for r in results)
    avg_tps = total_tokens / total_time if total_time > 0 else 0
    
    print(f"📊 TOTALES")
    print(f"   Tests ejecutados: {len(results)}")
    print(f"   ✅ Pasaron:       {total_passed} ({total_passed/len(results)*100:.1f}%)")
    print(f"   ❌ Fallaron:      {total_failed} ({total_failed/len(results)*100:.1f}%)")
    print(f"   ⏱️  Tiempo total:  {total_time:.2f}s")
    print(f"   📝 Tokens totales: {total_tokens}")
    print(f"   🚀 Promedio:      {avg_tps:.1f} tokens/segundo")
    print("=" * 70)
    
    # Tests fallidos
    failed = [r for r in results if not r.passed]
    if failed:
        print("\n⚠️  TESTS FALLIDOS:")
        print("-" * 70)
        for r in failed:
            print(f"\n❌ {r.name}")
            if r.error:
                print(f"   Error: {r.error}")
            else:
                print(f"   Response: {r.response[:150]}...")

# =============================================================================
# Main
# =============================================================================

def main():
    parser = argparse.ArgumentParser(description="BitNet Benchmark Suite")
    parser.add_argument("--url", default=DEFAULT_URL, help=f"URL del servidor (default: {DEFAULT_URL})")
    parser.add_argument("--verbose", "-v", action="store_true", help="Mostrar respuestas completas")
    args = parser.parse_args()
    
    client = BitNetClient(args.url)
    
    print(f"\n🔗 Conectando a {args.url}...")
    
    if not client.health_check():
        print(f"❌ Error: No se puede conectar a {args.url}")
        print("   Asegúrate de que neuro-bitnet está corriendo:")
        print("   docker-compose up -d")
        exit(1)
    
    print("✅ Servidor disponible")
    
    results = run_tests(client, verbose=args.verbose)
    print_summary(results)

if __name__ == "__main__":
    main()
