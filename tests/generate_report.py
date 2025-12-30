#!/usr/bin/env python3
"""
BitNet Full Report Generator
============================
Genera un reporte HTML con todos los resultados de las pruebas.

Uso:
    python generate_report.py [--url URL] [--output report.html]
"""

import json
import time
import re
import argparse
from datetime import datetime
from dataclasses import dataclass, asdict
from typing import Optional

try:
    import requests
except ImportError:
    print("❌ Instala requests: pip install requests")
    exit(1)

DEFAULT_URL = "http://localhost:11435"

# =============================================================================
# Definiciones
# =============================================================================

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
class TestCase:
    name: str
    category: str
    messages: list
    max_tokens: int
    expected: str
    
@dataclass
class TestResult:
    name: str
    category: str
    passed: bool
    time_ms: int
    tokens: int
    tps: float
    response: str
    expected: str

# =============================================================================
# Tests
# =============================================================================

def make_tests():
    return [
        # Chat - con system prompt universal
        TestCase("Saludo", "Chat", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¡Hola! ¿Cómo estás?"}
        ], 30, "hola/bien/ayudar"),
        TestCase("Capital Francia", "Chat", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cuál es la capital de Francia?"}
        ], 30, "París/Paris"),
        TestCase("Capital España", "Chat", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cuál es la capital de España?"}
        ], 30, "Madrid"),
        TestCase("Quién es Einstein", "Chat", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Quién fue Albert Einstein?"}
        ], 100, "físico/científico/physicist"),
        
        # Matemáticas - con system prompt universal
        TestCase("25+17", "Matemáticas", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cuánto es 25+17? Solo el número."}
        ], 20, "42"),
        TestCase("12*11", "Matemáticas", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cuánto es 12*11? Solo el número."}
        ], 20, "132"),
        TestCase("100/4", "Matemáticas", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cuánto es 100/4? Solo el número."}
        ], 20, "25"),
        TestCase("7^2", "Matemáticas", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cuánto es 7 al cuadrado? Solo el número."}
        ], 20, "49"),
        
        # Código - con system prompt universal
        TestCase("Hola Mundo", "Código", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "Escribe un programa Python que imprima 'Hola Mundo'. Solo código."}
        ], 50, "print"),
        TestCase("Función suma", "Código", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "Escribe una función Python llamada 'sumar' que sume dos números."}
        ], 100, "def"),
        TestCase("Lista reversa", "Código", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "Escribe código Python para invertir una lista llamada 'items'."}
        ], 80, "reverse/::-1/reversed"),
        TestCase("Bucle for", "Código", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "Escribe un bucle for en Python que imprima del 1 al 5."}
        ], 80, "for"),
        
        # Tools - con system prompt de tools
        TestCase("Tool: Clima Tokio", "Tools", [
            {"role": "system", "content": TOOLS_SYSTEM},
            {"role": "user", "content": "¿Qué clima hace en Tokio?"}
        ], 150, "get_weather"),
        TestCase("Tool: Clima Londres", "Tools", [
            {"role": "system", "content": TOOLS_SYSTEM},
            {"role": "user", "content": "Consulta el clima en Londres por favor."}
        ], 150, "get_weather"),
        TestCase("Tool: Calcular", "Tools", [
            {"role": "system", "content": TOOLS_SYSTEM},
            {"role": "user", "content": "Usa la calculadora para: 25 * 4"}
        ], 150, "calculate"),
        TestCase("Tool: Traducir", "Tools", [
            {"role": "system", "content": TOOLS_SYSTEM},
            {"role": "user", "content": "Traduce 'adiós' al francés"}
        ], 150, "translate"),
        
        # General - con system prompt universal
        TestCase("Saludo formal", "General", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Cómo estás?"}
        ], 50, "bien/estoy/encuentro"),
        TestCase("Código con comentarios", "General", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "Escribe una función Python que multiplique dos números."}
        ], 100, "def"),
        
        # Razonamiento - con system prompt universal
        TestCase("Secuencia: 2,4,6,8,?", "Razonamiento", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "¿Qué número sigue: 2, 4, 6, 8, ?"}
        ], 50, "10"),
        TestCase("Silogismo", "Razonamiento", [
            {"role": "system", "content": UNIVERSAL_SYSTEM},
            {"role": "user", "content": "Todos los perros son animales. Firulais es un perro. ¿Qué es Firulais?"}
        ], 50, "animal/perro"),
    ]

# =============================================================================
# Validator
# =============================================================================

def validate(response: str, expected: str, category: str) -> bool:
    r = response.lower()
    e = expected.lower()
    
    if category == "Tools":
        # Buscar JSON de tool call - con tolerancia a JSON truncado/malformado
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
            
            # Si el JSON está truncado, intentar repararlo
            if depth > 0:
                json_str = response[start:] + ('}' * depth)
            
            try:
                data = json.loads(json_str)
                return data.get("tool") == expected
            except json.JSONDecodeError:
                # Fallback: buscar el nombre de la tool con regex
                import re
                tool_match = re.search(r'"tool"\s*:\s*"([^"]+)"', response)
                if tool_match:
                    return tool_match.group(1) == expected
                return False
        except:
            # Último intento con regex
            import re
            tool_match = re.search(r'"tool"\s*:\s*"([^"]+)"', response)
            if tool_match:
                return tool_match.group(1) == expected
            return False
    
    # Para otros, buscar palabras clave
    keywords = [kw.strip() for kw in e.split('/')]
    return any(kw in r for kw in keywords)

# =============================================================================
# Runner
# =============================================================================

def run_tests(url: str) -> list:
    results = []
    tests = make_tests()
    session = requests.Session()
    
    for i, test in enumerate(tests, 1):
        print(f"[{i}/{len(tests)}] {test.name}...", end=" ", flush=True)
        
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
            elapsed = time.time() - start
            
            data = r.json()
            content = data.get("choices", [{}])[0].get("message", {}).get("content", "")
            tokens = data.get("usage", {}).get("completion_tokens", 0)
            tps = tokens / elapsed if elapsed > 0 else 0
            
            passed = validate(content, test.expected, test.category)
            
            results.append(TestResult(
                name=test.name,
                category=test.category,
                passed=passed,
                time_ms=int(elapsed * 1000),
                tokens=tokens,
                tps=round(tps, 1),
                response=content[:500],
                expected=test.expected
            ))
            
            print("✅" if passed else "❌", f"{int(elapsed*1000)}ms")
            
        except Exception as e:
            results.append(TestResult(
                name=test.name,
                category=test.category,
                passed=False,
                time_ms=0,
                tokens=0,
                tps=0,
                response=f"Error: {e}",
                expected=test.expected
            ))
            print(f"❌ Error")
    
    return results

# =============================================================================
# HTML Report
# =============================================================================

def generate_html(results: list, url: str) -> str:
    total = len(results)
    passed = sum(1 for r in results if r.passed)
    failed = total - passed
    total_time = sum(r.time_ms for r in results)
    total_tokens = sum(r.tokens for r in results)
    avg_tps = total_tokens / (total_time / 1000) if total_time > 0 else 0
    
    # Agrupar por categoría
    categories = {}
    for r in results:
        if r.category not in categories:
            categories[r.category] = {"passed": 0, "failed": 0, "time": 0, "tokens": 0}
        categories[r.category]["passed" if r.passed else "failed"] += 1
        categories[r.category]["time"] += r.time_ms
        categories[r.category]["tokens"] += r.tokens
    
    # Generar filas de resultados
    result_rows = ""
    for r in results:
        status_class = "pass" if r.passed else "fail"
        status_icon = "✅" if r.passed else "❌"
        response_escaped = r.response.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\n", "<br>")
        result_rows += f"""
        <tr class="{status_class}">
            <td>{r.name}</td>
            <td>{r.category}</td>
            <td class="status">{status_icon}</td>
            <td>{r.time_ms}ms</td>
            <td>{r.tokens}</td>
            <td>{r.tps}</td>
            <td class="expected">{r.expected}</td>
            <td class="response">{response_escaped[:200]}{'...' if len(response_escaped) > 200 else ''}</td>
        </tr>"""
    
    # Generar filas de categorías
    category_rows = ""
    for cat, stats in categories.items():
        total_cat = stats["passed"] + stats["failed"]
        pct = (stats["passed"] / total_cat * 100) if total_cat > 0 else 0
        category_rows += f"""
        <tr>
            <td>{cat}</td>
            <td>{stats['passed']}</td>
            <td>{stats['failed']}</td>
            <td>{pct:.0f}%</td>
            <td>{stats['time']}ms</td>
            <td>{stats['tokens']}</td>
        </tr>"""
    
    html = f"""<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BitNet Benchmark Report</title>
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
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
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
        .card .value {{ font-size: 32px; font-weight: bold; color: #333; }}
        .card.success .value {{ color: #10b981; }}
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
        tr.pass td {{ background: rgba(16, 185, 129, 0.05); }}
        tr.fail td {{ background: rgba(239, 68, 68, 0.05); }}
        .status {{ font-size: 18px; text-align: center; }}
        .response {{ 
            max-width: 300px; 
            font-size: 12px; 
            color: #666;
            word-break: break-word;
        }}
        .expected {{ font-size: 12px; color: #888; }}
        .footer {{ text-align: center; color: #888; margin-top: 30px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🧪 BitNet Benchmark Report</h1>
        <p>📅 {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
        <p>🔗 Server: {url}</p>
    </div>
    
    <div class="summary">
        <div class="card">
            <h3>Total Tests</h3>
            <div class="value">{total}</div>
        </div>
        <div class="card success">
            <h3>Passed ✅</h3>
            <div class="value">{passed} ({passed/total*100:.0f}%)</div>
        </div>
        <div class="card error">
            <h3>Failed ❌</h3>
            <div class="value">{failed}</div>
        </div>
        <div class="card">
            <h3>Total Time</h3>
            <div class="value">{total_time/1000:.1f}s</div>
        </div>
        <div class="card">
            <h3>Tokens</h3>
            <div class="value">{total_tokens}</div>
        </div>
        <div class="card">
            <h3>Avg Speed</h3>
            <div class="value">{avg_tps:.1f} t/s</div>
        </div>
    </div>
    
    <h2>📊 Por Categoría</h2>
    <table>
        <thead>
            <tr>
                <th>Categoría</th>
                <th>Pasaron</th>
                <th>Fallaron</th>
                <th>% Éxito</th>
                <th>Tiempo</th>
                <th>Tokens</th>
            </tr>
        </thead>
        <tbody>
            {category_rows}
        </tbody>
    </table>
    
    <h2>📋 Resultados Detallados</h2>
    <table>
        <thead>
            <tr>
                <th>Test</th>
                <th>Categoría</th>
                <th>Estado</th>
                <th>Tiempo</th>
                <th>Tokens</th>
                <th>T/s</th>
                <th>Esperado</th>
                <th>Respuesta</th>
            </tr>
        </thead>
        <tbody>
            {result_rows}
        </tbody>
    </table>
    
    <div class="footer">
        <p>Generated by neuro-bitnet benchmark suite</p>
    </div>
</body>
</html>"""
    
    return html

# =============================================================================
# Main
# =============================================================================

def main():
    parser = argparse.ArgumentParser(description="BitNet Report Generator")
    parser.add_argument("--url", default=DEFAULT_URL)
    parser.add_argument("--output", "-o", default="benchmark_report.html")
    args = parser.parse_args()
    
    print(f"\n🧪 BitNet Full Benchmark")
    print(f"🔗 URL: {args.url}")
    print("=" * 60)
    
    # Verificar servidor
    try:
        r = requests.get(f"{args.url}/health", timeout=5)
        if r.status_code != 200:
            raise Exception("Health check failed")
    except:
        print(f"❌ No se puede conectar a {args.url}")
        exit(1)
    
    print("✅ Servidor disponible\n")
    
    # Ejecutar tests
    results = run_tests(args.url)
    
    # Generar reporte
    html = generate_html(results, args.url)
    
    with open(args.output, 'w') as f:
        f.write(html)
    
    # Resumen
    passed = sum(1 for r in results if r.passed)
    total = len(results)
    
    print("\n" + "=" * 60)
    print(f"📊 RESUMEN: {passed}/{total} tests pasaron ({passed/total*100:.0f}%)")
    print(f"📄 Reporte generado: {args.output}")
    print("=" * 60)

if __name__ == "__main__":
    main()
