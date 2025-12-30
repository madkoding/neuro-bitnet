#!/usr/bin/env python3
"""
BitNet Full Report Generator
============================
Genera un reporte HTML con todos los resultados de las pruebas.
Ejecuta cada prueba múltiples veces para medir precisión real.

Uso:
    python generate_report.py [--url URL] [--output report.html] [--runs N]
"""

import json
import time
import re
import argparse
from datetime import datetime
from dataclasses import dataclass, field
from typing import Optional, List

try:
    import requests
except ImportError:
    print("❌ Instala requests: pip install requests")
    exit(1)

DEFAULT_URL = "http://localhost:11435"
DEFAULT_RUNS = 10  # Número de veces que se ejecuta cada test

# =============================================================================
# System Prompts específicos por categoría (simples, sin conocimiento previo)
# =============================================================================

SYSTEM_CHAT = """Eres un asistente conversacional. Responde de forma natural y amigable.
Responde en el mismo idioma que el usuario."""

SYSTEM_MATH = """Eres una calculadora. Responde SOLO con el número resultante.
No expliques, no muestres pasos. Solo el número."""

SYSTEM_CODE = """Eres un programador experto. Responde SOLO con código.
No expliques, no agregues comentarios innecesarios. Solo código funcional."""

SYSTEM_TOOLS = """Eres un asistente con acceso a herramientas.

HERRAMIENTAS:
- get_weather(location): Clima de una ciudad
- calculate(expression): Calcular expresión matemática
- translate(text, to_language): Traducir texto

RESPONDE SOLO CON JSON: {"tool": "nombre", "arguments": {"param": "valor"}}"""

SYSTEM_REASONING = """Eres un asistente de razonamiento lógico.
Analiza el problema y da una respuesta concisa."""

SYSTEM_GENERAL = """Eres un asistente de IA útil y preciso.
Responde de forma clara y concisa."""

# =============================================================================
# Definiciones
# =============================================================================

@dataclass
class TestCase:
    name: str
    category: str
    system: str
    user: str
    max_tokens: int
    expected: str

@dataclass
class TestResult:
    name: str
    category: str
    runs: int
    passed: int
    accuracy: float
    avg_time_ms: int
    avg_tokens: float
    avg_tps: float
    responses: List[str] = field(default_factory=list)
    expected: str = ""

# =============================================================================
# Tests por categoría
# =============================================================================

def make_tests():
    return [
        # === CHAT ===
        TestCase("Saludo", "Chat", SYSTEM_CHAT,
                 "¡Hola! ¿Cómo estás?",
                 50, "hola/bien/ayud"),
        TestCase("Capital Francia", "Chat", SYSTEM_CHAT,
                 "¿Cuál es la capital de Francia?",
                 30, "París/Paris"),
        TestCase("Capital España", "Chat", SYSTEM_CHAT,
                 "¿Cuál es la capital de España?",
                 30, "Madrid"),
        TestCase("Quién es Einstein", "Chat", SYSTEM_CHAT,
                 "¿Quién fue Albert Einstein?",
                 100, "físico/científico/physicist/ciencia"),
        
        # === MATEMÁTICAS ===
        TestCase("25+17", "Matemáticas", SYSTEM_MATH,
                 "25+17", 20, "42"),
        TestCase("12*11", "Matemáticas", SYSTEM_MATH,
                 "12*11", 20, "132"),
        TestCase("100/4", "Matemáticas", SYSTEM_MATH,
                 "100/4", 20, "25"),
        TestCase("7^2", "Matemáticas", SYSTEM_MATH,
                 "7 al cuadrado", 20, "49"),
        
        # === CÓDIGO ===
        TestCase("Hola Mundo", "Código", SYSTEM_CODE,
                 "print Hola Mundo en Python",
                 50, "print"),
        TestCase("Función suma", "Código", SYSTEM_CODE,
                 "función Python que sume dos números",
                 100, "def"),
        TestCase("Lista reversa", "Código", SYSTEM_CODE,
                 "código Python para invertir una lista",
                 100, "reverse/::-1/reversed"),
        TestCase("Bucle for", "Código", SYSTEM_CODE,
                 "bucle for en Python del 1 al 5",
                 100, "for"),
        
        # === TOOLS ===
        TestCase("Tool: Clima", "Tools", SYSTEM_TOOLS,
                 "clima en Tokio",
                 150, "get_weather"),
        TestCase("Tool: Calcular", "Tools", SYSTEM_TOOLS,
                 "calcula 25*4",
                 150, "calculate"),
        TestCase("Tool: Traducir", "Tools", SYSTEM_TOOLS,
                 "traduce 'hola' al inglés",
                 150, "translate"),
        
        # === RAZONAMIENTO ===
        TestCase("Secuencia", "Razonamiento", SYSTEM_REASONING,
                 "¿Qué número sigue: 2, 4, 6, 8, ?",
                 50, "10"),
        TestCase("Silogismo", "Razonamiento", SYSTEM_REASONING,
                 "Si todos los gatos son animales, y Michi es un gato, ¿qué es Michi?",
                 50, "animal/gato"),
        TestCase("Lógica", "Razonamiento", SYSTEM_REASONING,
                 "Si llueve, el suelo se moja. Está lloviendo. ¿Cómo está el suelo?",
                 50, "mojado/wet"),
        
        # === GENERAL ===
        TestCase("Saludo formal", "General", SYSTEM_GENERAL,
                 "Buenos días",
                 50, "buenos/días/hola/salud"),
        TestCase("Despedida", "General", SYSTEM_GENERAL,
                 "Adiós, gracias por tu ayuda",
                 50, "adiós/hasta/gusto/nada"),
    ]

# =============================================================================
# Validadores
# =============================================================================

def validate(response: str, expected: str, category: str) -> bool:
    r = response.lower()
    
    if category == "Tools":
        # Buscar tool call en JSON
        tool_match = re.search(r'"tool"\s*:\s*"([^"]+)"', response)
        if tool_match:
            return tool_match.group(1) == expected
        return False
    
    # Para otros, buscar palabras clave
    keywords = [kw.strip().lower() for kw in expected.split('/')]
    return any(kw in r for kw in keywords)

# =============================================================================
# Runner con múltiples ejecuciones
# =============================================================================

def run_single_test(session, url: str, test: TestCase) -> tuple:
    """Ejecuta un solo test y retorna (passed, time_ms, tokens, tps, response)"""
    try:
        start = time.time()
        r = session.post(
            f"{url}/v1/chat/completions",
            json={
                "model": "bitnet",
                "messages": [
                    {"role": "system", "content": test.system},
                    {"role": "user", "content": test.user}
                ],
                "max_tokens": test.max_tokens,
                "temperature": 0.3
            },
            timeout=60
        )
        elapsed = time.time() - start
        
        data = r.json()
        content = data.get("choices", [{}])[0].get("message", {}).get("content", "")
        tokens = data.get("usage", {}).get("completion_tokens", 0)
        tps = tokens / elapsed if elapsed > 0 else 0
        
        passed = validate(content, test.expected, test.category)
        
        return (passed, int(elapsed * 1000), tokens, tps, content[:200])
    except Exception as e:
        return (False, 0, 0, 0, f"Error: {e}")

def run_tests(url: str, runs: int = DEFAULT_RUNS) -> list:
    """Ejecuta cada test múltiples veces y calcula precisión"""
    results = []
    tests = make_tests()
    session = requests.Session()
    
    total_runs = len(tests) * runs
    current = 0
    
    print(f"\n📊 Ejecutando {len(tests)} tests × {runs} veces = {total_runs} ejecuciones\n")
    
    for test in tests:
        passed_count = 0
        times = []
        tokens_list = []
        tps_list = []
        responses = []
        
        print(f"[{test.name}] ", end="", flush=True)
        
        for run in range(runs):
            current += 1
            passed, time_ms, tokens, tps, response = run_single_test(session, url, test)
            
            if passed:
                passed_count += 1
                print("✓", end="", flush=True)
            else:
                print("✗", end="", flush=True)
            
            times.append(time_ms)
            tokens_list.append(tokens)
            tps_list.append(tps)
            responses.append(response)
        
        accuracy = (passed_count / runs) * 100
        avg_time = sum(times) / len(times) if times else 0
        avg_tokens = sum(tokens_list) / len(tokens_list) if tokens_list else 0
        avg_tps = sum(tps_list) / len(tps_list) if tps_list else 0
        
        print(f" → {accuracy:.0f}% ({passed_count}/{runs})")
        
        results.append(TestResult(
            name=test.name,
            category=test.category,
            runs=runs,
            passed=passed_count,
            accuracy=accuracy,
            avg_time_ms=int(avg_time),
            avg_tokens=round(avg_tokens, 1),
            avg_tps=round(avg_tps, 1),
            responses=responses[:3],  # Guardar solo 3 ejemplos
            expected=test.expected
        ))
    
    return results

# =============================================================================
# HTML Report
# =============================================================================

def generate_html(results: list, url: str, runs: int) -> str:
    total_tests = len(results)
    total_runs = total_tests * runs
    total_passed = sum(r.passed for r in results)
    overall_accuracy = (total_passed / total_runs) * 100 if total_runs > 0 else 0
    total_time = sum(r.avg_time_ms * runs for r in results)
    total_tokens = sum(r.avg_tokens * runs for r in results)
    avg_tps = sum(r.avg_tps for r in results) / len(results) if results else 0
    
    # Tests con 100% accuracy
    perfect_tests = sum(1 for r in results if r.accuracy == 100)
    
    # Agrupar por categoría
    categories = {}
    for r in results:
        if r.category not in categories:
            categories[r.category] = {"tests": 0, "passed": 0, "runs": 0, "time": 0}
        categories[r.category]["tests"] += 1
        categories[r.category]["passed"] += r.passed
        categories[r.category]["runs"] += r.runs
        categories[r.category]["time"] += r.avg_time_ms
    
    # Generar filas de resultados
    result_rows = ""
    for r in results:
        # Color según precisión
        if r.accuracy == 100:
            status_class = "perfect"
            status_icon = "🎯"
        elif r.accuracy >= 70:
            status_class = "good"
            status_icon = "✅"
        elif r.accuracy >= 40:
            status_class = "warning"
            status_icon = "⚠️"
        else:
            status_class = "bad"
            status_icon = "❌"
        
        # Muestra de respuestas
        sample_responses = "<br>".join([
            f"<small>{resp[:100]}{'...' if len(resp) > 100 else ''}</small>"
            for resp in r.responses[:2]
        ])
        
        result_rows += f"""
        <tr class="{status_class}">
            <td>{r.name}</td>
            <td>{r.category}</td>
            <td class="accuracy">{status_icon} {r.accuracy:.0f}%</td>
            <td>{r.passed}/{r.runs}</td>
            <td>{r.avg_time_ms}ms</td>
            <td>{r.avg_tokens}</td>
            <td>{r.avg_tps}</td>
            <td class="expected">{r.expected}</td>
            <td class="response">{sample_responses}</td>
        </tr>"""
    
    # Generar filas de categorías
    category_rows = ""
    for cat, stats in categories.items():
        pct = (stats["passed"] / stats["runs"] * 100) if stats["runs"] > 0 else 0
        avg_time = stats["time"] / stats["tests"] if stats["tests"] > 0 else 0
        
        if pct == 100:
            pct_class = "perfect"
        elif pct >= 70:
            pct_class = "good"
        elif pct >= 40:
            pct_class = "warning"
        else:
            pct_class = "bad"
        
        category_rows += f"""
        <tr>
            <td>{cat}</td>
            <td>{stats['tests']}</td>
            <td>{stats['passed']}/{stats['runs']}</td>
            <td class="{pct_class}">{pct:.1f}%</td>
            <td>{avg_time:.0f}ms</td>
        </tr>"""
    
    html = f"""<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BitNet Benchmark Report - Precisión</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1400px; 
            margin: 0 auto; 
            padding: 20px;
            background: #f5f5f5;
        }}
        h1, h2 {{ color: #333; }}
        .header {{ 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white; 
            padding: 30px; 
            border-radius: 10px;
            margin-bottom: 20px;
        }}
        .header h1 {{ margin: 0 0 10px 0; }}
        .header p {{ margin: 5px 0; opacity: 0.9; }}
        .summary {{ 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
            gap: 15px;
            margin-bottom: 20px;
        }}
        .card {{
            background: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .card h3 {{ margin: 0 0 10px 0; color: #666; font-size: 14px; }}
        .card .value {{ font-size: 28px; font-weight: bold; color: #333; }}
        .card.success .value {{ color: #10b981; }}
        .card.warning .value {{ color: #f59e0b; }}
        .card.error .value {{ color: #ef4444; }}
        table {{ 
            width: 100%; 
            border-collapse: collapse;
            background: white;
            border-radius: 10px;
            overflow: hidden;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            margin-bottom: 20px;
        }}
        th, td {{ 
            padding: 12px 15px; 
            text-align: left; 
            border-bottom: 1px solid #eee;
        }}
        th {{ background: #f8f9fa; font-weight: 600; color: #666; }}
        tr:hover {{ background: #f8f9fa; }}
        tr.perfect td {{ background: rgba(16, 185, 129, 0.1); }}
        tr.good td {{ background: rgba(16, 185, 129, 0.05); }}
        tr.warning td {{ background: rgba(245, 158, 11, 0.1); }}
        tr.bad td {{ background: rgba(239, 68, 68, 0.1); }}
        .accuracy {{ font-weight: bold; }}
        td.perfect {{ color: #10b981; }}
        td.good {{ color: #10b981; }}
        td.warning {{ color: #f59e0b; }}
        td.bad {{ color: #ef4444; }}
        .response {{ 
            max-width: 250px; 
            font-size: 11px; 
            color: #666;
            word-break: break-word;
        }}
        .expected {{ font-size: 12px; color: #888; max-width: 100px; }}
        .footer {{ text-align: center; color: #888; margin-top: 30px; }}
        .legend {{
            display: flex;
            gap: 20px;
            margin-bottom: 15px;
            font-size: 14px;
        }}
        .legend span {{ display: flex; align-items: center; gap: 5px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🧪 BitNet Benchmark Report - Análisis de Precisión</h1>
        <p>📅 {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
        <p>🔗 Server: {url}</p>
        <p>🔄 Cada test ejecutado {runs} veces para medir consistencia</p>
    </div>
    
    <div class="summary">
        <div class="card {'success' if overall_accuracy >= 80 else 'warning' if overall_accuracy >= 60 else 'error'}">
            <h3>Precisión Global</h3>
            <div class="value">{overall_accuracy:.1f}%</div>
        </div>
        <div class="card">
            <h3>Tests Totales</h3>
            <div class="value">{total_tests}</div>
        </div>
        <div class="card">
            <h3>Ejecuciones</h3>
            <div class="value">{total_runs}</div>
        </div>
        <div class="card success">
            <h3>Tests 100% ✓</h3>
            <div class="value">{perfect_tests}</div>
        </div>
        <div class="card">
            <h3>Tiempo Total</h3>
            <div class="value">{total_time/1000:.1f}s</div>
        </div>
        <div class="card">
            <h3>Vel. Promedio</h3>
            <div class="value">{avg_tps:.1f} t/s</div>
        </div>
    </div>
    
    <h2>📊 Precisión por Categoría</h2>
    <table>
        <thead>
            <tr>
                <th>Categoría</th>
                <th>Tests</th>
                <th>Pasaron</th>
                <th>Precisión</th>
                <th>Tiempo Prom.</th>
            </tr>
        </thead>
        <tbody>
            {category_rows}
        </tbody>
    </table>
    
    <h2>📋 Resultados Detallados</h2>
    <div class="legend">
        <span>🎯 100%</span>
        <span>✅ ≥70%</span>
        <span>⚠️ ≥40%</span>
        <span>❌ &lt;40%</span>
    </div>
    <table>
        <thead>
            <tr>
                <th>Test</th>
                <th>Categoría</th>
                <th>Precisión</th>
                <th>Pasaron</th>
                <th>Tiempo Prom.</th>
                <th>Tokens Prom.</th>
                <th>T/s</th>
                <th>Esperado</th>
                <th>Muestras</th>
            </tr>
        </thead>
        <tbody>
            {result_rows}
        </tbody>
    </table>
    
    <div class="footer">
        <p>Generated by neuro-bitnet benchmark suite</p>
        <p>Cada test se ejecutó {runs} veces para medir precisión estadística</p>
    </div>
</body>
</html>"""
    
    return html

# =============================================================================
# Main
# =============================================================================

def main():
    parser = argparse.ArgumentParser(description="BitNet Report Generator con análisis de precisión")
    parser.add_argument("--url", default=DEFAULT_URL)
    parser.add_argument("--output", "-o", default="benchmark_report.html")
    parser.add_argument("--runs", "-r", type=int, default=DEFAULT_RUNS,
                        help=f"Número de veces que se ejecuta cada test (default: {DEFAULT_RUNS})")
    args = parser.parse_args()
    
    print(f"\n🧪 BitNet Benchmark - Análisis de Precisión")
    print(f"🔗 URL: {args.url}")
    print(f"🔄 Runs por test: {args.runs}")
    print("=" * 60)
    
    # Verificar servidor
    try:
        r = requests.get(f"{args.url}/health", timeout=5)
        if r.status_code != 200:
            raise Exception("Health check failed")
    except:
        print(f"❌ No se puede conectar a {args.url}")
        exit(1)
    
    print("✅ Servidor disponible")
    
    # Ejecutar tests
    results = run_tests(args.url, args.runs)
    
    # Generar reporte
    html = generate_html(results, args.url, args.runs)
    
    with open(args.output, 'w') as f:
        f.write(html)
    
    # Resumen
    total_passed = sum(r.passed for r in results)
    total_runs = sum(r.runs for r in results)
    overall_accuracy = (total_passed / total_runs) * 100 if total_runs > 0 else 0
    perfect = sum(1 for r in results if r.accuracy == 100)
    
    print("\n" + "=" * 60)
    print(f"📊 PRECISIÓN GLOBAL: {overall_accuracy:.1f}%")
    print(f"🎯 Tests con 100%: {perfect}/{len(results)}")
    print(f"📄 Reporte generado: {args.output}")
    print("=" * 60)

if __name__ == "__main__":
    main()
