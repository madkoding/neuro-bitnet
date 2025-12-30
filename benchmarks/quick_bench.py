#!/usr/bin/env python3
"""
BitNet Quick Benchmark
======================
Benchmark rápido que muestra una tabla comparativa clara.

Ejecuta tests seleccionados y muestra:
- Nombre del test
- Categoría (con/sin tools)
- Tiempo de respuesta
- Resultado (PASS/FAIL)
- Respuesta resumida

Uso:
    python quick_bench.py [--url URL]
"""

import json
import time
import re
import argparse
from dataclasses import dataclass
from typing import Optional

try:
    import requests
except ImportError:
    print("❌ Instala requests: pip install requests")
    exit(1)

# =============================================================================
# Configuración
# =============================================================================

DEFAULT_URL = "http://localhost:11435"

# System prompt universal
UNIVERSAL_SYSTEM = """Eres un asistente de IA preciso y útil. Sigue estas reglas:

IDIOMA: Responde en el mismo idioma que el usuario.

FORMATO:
- Preguntas simples (capitales, datos): responde solo con el dato
- Matemáticas: muestra el resultado numérico primero
- Código: proporciona código funcional y limpio

CONOCIMIENTO:
- París es la capital de Francia
- Madrid es la capital de España
- Londres es la capital de Reino Unido"""

# System prompt para tools
TOOLS_SYSTEM = """Eres un asistente de IA preciso con acceso a herramientas.

HERRAMIENTAS DISPONIBLES:
- get_weather(location, unit): Obtener clima de una ciudad
- calculate(expression): Calcular expresiones matemáticas
- translate(text, to_language): Traducir texto

CUÁNDO USAR HERRAMIENTAS:
✅ USA herramientas para: clima actual, cálculos complejos, traducciones
❌ NO uses herramientas para: conocimiento general, matemáticas simples

FORMATO: Responde SOLO con JSON: {"tool": "nombre", "arguments": {"param": "valor"}}"""

@dataclass
class Test:
    name: str
    category: str
    messages: list
    max_tokens: int
    validator: callable
    expected_desc: str

@dataclass
class Result:
    test: Test
    passed: bool
    time_ms: int
    tokens: int
    response: str
    error: Optional[str] = None

# =============================================================================
# Tests
# =============================================================================

QUICK_TESTS = [
    # Sin Tools
    Test(
        name="Saludo básico",
        category="Chat",
        messages=[
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¡Hola! ¿Cómo estás?"}
        ],
        max_tokens=30,
        validator=lambda r: len(r) > 5,
        expected_desc="Respuesta > 5 chars"
    ),
    Test(
        name="Capital de Francia",
        category="Chat",
        messages=[
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cuál es la capital de Francia?"}
        ],
        max_tokens=20,
        validator=lambda r: "paris" in r.lower() or "parís" in r.lower(),
        expected_desc='Contiene "París"'
    ),
    Test(
        name="Suma 25+17",
        category="Matemáticas",
        messages=[
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cuánto es 25 + 17? Responde solo con el número."}
        ],
        max_tokens=10,
        validator=lambda r: "42" in r,
        expected_desc='Contiene "42"'
    ),
    Test(
        name="Python Hello World",
        category="Código",
        messages=[
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "Escribe un programa Python que imprima 'Hola Mundo'. Solo código."}
        ],
        max_tokens=50,
        validator=lambda r: "print" in r.lower(),
        expected_desc='Contiene "print"'
    ),
    Test(
        name="Capital de España",
        category="Chat",
        messages=[
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cuál es la capital de España?"}
        ],
        max_tokens=30,
        validator=lambda r: "madrid" in r.lower(),
        expected_desc='Contiene "Madrid"'
    ),
    
    # Con Tools
    Test(
        name="Tool: Clima",
        category="Tools",
        messages=[
            {"role": "system", "content": TOOLS_SYSTEM},
            {"role": "user", "content": "¿Qué clima hace en Tokio?"}
        ],
        max_tokens=80,
        validator=lambda r: _is_tool_call(r, "get_weather"),
        expected_desc='JSON con tool="get_weather"'
    ),
    Test(
        name="Tool: Calcular",
        category="Tools",
        messages=[
            {"role": "system", "content": TOOLS_SYSTEM},
            {"role": "user", "content": "Calcula 15 multiplicado por 8"}
        ],
        max_tokens=80,
        validator=lambda r: _is_tool_call(r, "calculate"),
        expected_desc='JSON con tool="calculate"'
    ),
    Test(
        name="Tool: Traducir",
        category="Tools",
        messages=[
            {"role": "system", "content": TOOLS_SYSTEM},
            {"role": "user", "content": "Traduce 'hola' al inglés"}
        ],
        max_tokens=80,
        validator=lambda r: _is_tool_call(r, "translate"),
        expected_desc='JSON con tool="translate"'
    ),
    Test(
        name="Sin Tool: Conocimiento",
        category="Tools",
        messages=[
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Qué es Python?"}
        ],
        max_tokens=100,
        validator=lambda r: "python" in r.lower() or "programación" in r.lower() or "lenguaje" in r.lower(),
        expected_desc="Respuesta sobre Python"
    ),
]

# =============================================================================
# Helpers
# =============================================================================

def _is_tool_call(response: str, tool_name: str) -> bool:
    try:
        # Buscar JSON anidado (puede tener {} dentro de arguments)
        # Buscar desde { hasta el último }
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
        data = json.loads(json_str)
        return data.get("tool") == tool_name
    except:
        pass
    return False

def _has_tool_json(response: str) -> bool:
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

def truncate(text: str, length: int = 40) -> str:
    text = text.replace('\n', ' ').strip()
    if len(text) > length:
        return text[:length-3] + "..."
    return text

# =============================================================================
# Runner
# =============================================================================

def run_tests(url: str) -> list:
    results = []
    session = requests.Session()
    
    print(f"\n🔗 Servidor: {url}")
    print("=" * 100)
    
    for i, test in enumerate(QUICK_TESTS, 1):
        print(f"[{i}/{len(QUICK_TESTS)}] {test.name}...", end=" ", flush=True)
        
        try:
            start = time.time()
            r = session.post(
                f"{url}/v1/chat/completions",
                json={
                    "model": "bitnet",
                    "messages": test.messages,
                    "max_tokens": test.max_tokens,
                    "temperature": 0.3
                },
                timeout=60
            )
            elapsed_ms = int((time.time() - start) * 1000)
            
            data = r.json()
            content = data.get("choices", [{}])[0].get("message", {}).get("content", "")
            tokens = data.get("usage", {}).get("completion_tokens", 0)
            
            passed = test.validator(content)
            
            results.append(Result(
                test=test,
                passed=passed,
                time_ms=elapsed_ms,
                tokens=tokens,
                response=content
            ))
            
            status = "✅" if passed else "❌"
            print(f"{status} {elapsed_ms}ms")
            
        except Exception as e:
            results.append(Result(
                test=test,
                passed=False,
                time_ms=0,
                tokens=0,
                response="",
                error=str(e)
            ))
            print(f"❌ Error: {e}")
    
    return results

def print_table(results: list):
    """Imprime tabla comparativa."""
    
    print("\n")
    print("=" * 110)
    print("📊 TABLA COMPARATIVA DE RESULTADOS")
    print("=" * 110)
    
    # Header
    print(f"{'#':<3} {'Test':<25} {'Categoría':<10} {'Estado':<8} {'Tiempo':<10} {'Tokens':<8} {'Respuesta':<40}")
    print("-" * 110)
    
    # Rows
    for i, r in enumerate(results, 1):
        status = "✅ PASS" if r.passed else "❌ FAIL"
        time_str = f"{r.time_ms}ms" if r.time_ms else "N/A"
        response = truncate(r.response) if r.response else r.error or "N/A"
        
        print(f"{i:<3} {r.test.name:<25} {r.test.category:<10} {status:<8} {time_str:<10} {r.tokens:<8} {response:<40}")
    
    print("-" * 110)
    
    # Summary
    passed = sum(1 for r in results if r.passed)
    failed = len(results) - passed
    total_time = sum(r.time_ms for r in results)
    avg_time = total_time / len(results) if results else 0
    total_tokens = sum(r.tokens for r in results)
    
    print(f"\n📈 RESUMEN")
    print(f"   Total tests:     {len(results)}")
    print(f"   ✅ Pasaron:      {passed} ({passed/len(results)*100:.0f}%)")
    print(f"   ❌ Fallaron:     {failed} ({failed/len(results)*100:.0f}%)")
    print(f"   ⏱️  Tiempo total: {total_time}ms ({total_time/1000:.1f}s)")
    print(f"   ⏱️  Promedio:     {avg_time:.0f}ms por test")
    print(f"   📝 Tokens:       {total_tokens} total")
    
    # Por categoría
    print(f"\n📊 POR CATEGORÍA")
    categories = {}
    for r in results:
        cat = r.test.category
        if cat not in categories:
            categories[cat] = {"passed": 0, "failed": 0, "time": 0}
        if r.passed:
            categories[cat]["passed"] += 1
        else:
            categories[cat]["failed"] += 1
        categories[cat]["time"] += r.time_ms
    
    print(f"   {'Categoría':<12} {'Pasaron':<10} {'Fallaron':<10} {'Tiempo':<12}")
    print(f"   {'-'*44}")
    for cat, stats in categories.items():
        print(f"   {cat:<12} {stats['passed']:<10} {stats['failed']:<10} {stats['time']}ms")
    
    # Tests fallidos
    failed_tests = [r for r in results if not r.passed]
    if failed_tests:
        print(f"\n⚠️  TESTS FALLIDOS - DETALLES")
        print("-" * 80)
        for r in failed_tests:
            print(f"\n   ❌ {r.test.name}")
            print(f"      Esperado: {r.test.expected_desc}")
            print(f"      Recibido: {truncate(r.response, 60) if r.response else r.error}")
    
    print("\n" + "=" * 110)

# =============================================================================
# Main
# =============================================================================

def main():
    parser = argparse.ArgumentParser(description="BitNet Quick Benchmark")
    parser.add_argument("--url", default=DEFAULT_URL, help=f"URL del servidor")
    args = parser.parse_args()
    
    # Verificar servidor
    try:
        r = requests.get(f"{args.url}/health", timeout=5)
        if r.status_code != 200:
            raise Exception("Health check failed")
    except:
        print(f"❌ No se puede conectar a {args.url}")
        print("   Asegúrate de que el servidor está corriendo: docker-compose up -d")
        exit(1)
    
    print("\n🧪 BitNet Quick Benchmark")
    
    results = run_tests(args.url)
    print_table(results)

if __name__ == "__main__":
    main()
