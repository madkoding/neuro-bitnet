#!/usr/bin/env python3
"""
BitNet Full Report Generator
============================
Genera un reporte Markdown con todos los resultados de las pruebas.
Ejecuta cada prueba múltiples veces para medir precisión real.

Uso:
    python generate_report.py [--url URL] [--output report.md] [--runs N]
    python generate_report.py --rag  # Usar RAG con auto-learn
    python generate_report.py --compare  # Comparar LLM directo vs RAG inteligente
"""

import os
import sys
import json
import time
import re
import argparse
from datetime import datetime
from dataclasses import dataclass, field
from typing import Optional, List, Dict

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
Responde SOLO con la conclusión, sin repetir el problema.
Sé breve y directo."""

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
                 50, "mojado/wet/moja/húmedo"),
        
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
        # Buscar tool call en JSON - aceptar variaciones del nombre
        tool_match = re.search(r'"tool"\s*:\s*"([^"]+)"', response)
        if tool_match:
            tool_found = tool_match.group(1).lower()
            # Aceptar variaciones comunes
            expected_variations = {
                "get_weather": ["get_weather", "weather", "getweather"],
                "calculate": ["calculate", "calculator", "calc"],
                "translate": ["translate", "translator", "translation"],
            }
            expected_lower = expected.lower()
            valid_names = expected_variations.get(expected_lower, [expected_lower])
            return tool_found in valid_names
        return False
    
    # Para otros, buscar palabras clave
    keywords = [kw.strip().lower() for kw in expected.split('/')]
    return any(kw in r for kw in keywords)

# =============================================================================
# Sistema RAG (importación lazy)
# =============================================================================

_rag_system = None

def get_rag_system(url: str):
    """Obtiene o crea el sistema RAG con auto-learn"""
    global _rag_system
    if _rag_system is None:
        # Añadir directorio de scripts al path
        scripts_dir = os.path.join(os.path.dirname(__file__), '..', 'scripts')
        sys.path.insert(0, scripts_dir)
        
        from rag import RAGSystem
        _rag_system = RAGSystem(llm_url=url, auto_learn=True)
        print("🧠 Sistema RAG con auto-learn inicializado")
    return _rag_system

# =============================================================================
# Sistema RAG Inteligente (nuevo servidor persistente)
# =============================================================================

RAG_SERVER_URL = os.getenv("RAG_SERVER_URL", "http://localhost:11436")

def run_single_test_smart_rag(url: str, test: TestCase) -> tuple:
    """Ejecuta un solo test usando el RAG Server inteligente"""
    try:
        start = time.time()
        
        # El RAG server clasifica automáticamente y decide la estrategia
        response = requests.post(
            f"{RAG_SERVER_URL}/query",
            json={
                "question": test.user,
                "user_id": "benchmark"
            },
            timeout=120
        )
        response.raise_for_status()
        data = response.json()
        
        elapsed = time.time() - start
        content = data.get("answer", "")
        
        # Obtener info de clasificación
        classification = data.get("classification", {})
        timing = data.get("timing", {})
        
        # Estimar tokens (el RAG server no devuelve esto directamente)
        tokens = len(content.split()) * 1.3
        tps = tokens / elapsed if elapsed > 0 else 0
        
        passed = validate(content, test.expected, test.category)
        
        # Añadir info de estrategia usada
        strategy = classification.get("strategy", "unknown")
        response_info = f"[{strategy}] {content[:180]}"
        
        return (passed, int(elapsed * 1000), int(tokens), tps, response_info)
    except requests.exceptions.ConnectionError:
        return (False, 0, 0, 0, "Error: RAG Server no disponible")
    except Exception as e:
        return (False, 0, 0, 0, f"Error RAG Smart: {e}")

# =============================================================================
# Runner con múltiples ejecuciones
# =============================================================================

def run_single_test_rag(url: str, test: TestCase) -> tuple:
    """Ejecuta un solo test usando RAG con auto-learn"""
    try:
        rag = get_rag_system(url)
        start = time.time()
        
        # Categorías que requieren system prompt específico (no buscar en web)
        # Chat y General usan RAG con búsqueda web, el resto va directo al LLM
        direct_categories = ["Matemáticas", "Código", "Tools", "Razonamiento"]
        
        if test.category in direct_categories:
            # Bypass RAG: ir directo al LLM con el system prompt original
            system_prompt = test.system
        else:
            # Usar RAG con búsqueda web para conocimiento general
            system_prompt = None
        
        # Usar RAG query (que incluye auto-learn con búsqueda web si system_prompt=None)
        response = rag.query(test.user, stream=False, system_prompt=system_prompt)
        
        elapsed = time.time() - start
        # El RAG no da tokens directamente, estimamos
        tokens = len(response.split()) * 1.3  # Aprox tokens por palabras
        tps = tokens / elapsed if elapsed > 0 else 0
        
        passed = validate(response, test.expected, test.category)
        
        return (passed, int(elapsed * 1000), int(tokens), tps, response[:200])
    except Exception as e:
        return (False, 0, 0, 0, f"Error RAG: {e}")

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

def run_tests(url: str, runs: int = DEFAULT_RUNS, use_rag: bool = False, 
              use_smart_rag: bool = False, category_filter: str = None) -> list:
    """Ejecuta cada test múltiples veces y calcula precisión"""
    results = []
    all_tests = make_tests()
    
    # Filtrar por categoría si se especifica
    if category_filter:
        tests = [t for t in all_tests if t.category.lower() == category_filter.lower()]
        if not tests:
            print(f"❌ Categoría '{category_filter}' no encontrada.")
            print(f"   Categorías disponibles: {', '.join(set(t.category for t in all_tests))}")
            return []
    else:
        tests = all_tests
    
    session = requests.Session() if not (use_rag or use_smart_rag) else None
    
    total_runs = len(tests) * runs
    current = 0
    
    if use_smart_rag:
        mode_str = "RAG Inteligente 🧠✨"
    elif use_rag:
        mode_str = "RAG auto-learn 🧠"
    else:
        mode_str = "LLM directo"
    
    cat_str = f" [{category_filter}]" if category_filter else ""
    print(f"\n📊 Ejecutando {len(tests)} tests{cat_str} × {runs} veces = {total_runs} ejecuciones ({mode_str})\n")
    
    for test in tests:
        passed_count = 0
        times = []
        tokens_list = []
        tps_list = []
        responses = []
        
        print(f"[{test.name}] ", end="", flush=True)
        
        for run in range(runs):
            current += 1
            
            if use_smart_rag:
                passed, time_ms, tokens, tps, response = run_single_test_smart_rag(url, test)
            elif use_rag:
                passed, time_ms, tokens, tps, response = run_single_test_rag(url, test)
            else:
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
# Markdown Report
# =============================================================================

def generate_markdown(results: list, url: str, runs: int) -> str:
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
    
    # Construir markdown
    md = f"""# 🧪 BitNet Benchmark Report - Análisis de Precisión

**Fecha:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}  
**Server:** {url}  
**Ejecuciones:** Cada test ejecutado {runs} veces para medir consistencia

---

## 📊 Resumen Global

| Métrica | Valor |
|---------|-------|
| **Precisión Global** | **{overall_accuracy:.1f}%** |
| Tests Totales | {total_tests} |
| Ejecuciones | {total_runs} |
| Tests con 100% ✓ | {perfect_tests} |
| Tiempo Total | {total_time/1000:.1f}s |
| Velocidad Promedio | {avg_tps:.1f} t/s |

---

## 📊 Precisión por Categoría

| Categoría | Tests | Pasaron | Precisión | Tiempo Prom. |
|-----------|-------|---------|-----------|--------------|
"""
    
    # Agregar filas de categorías
    for cat, stats in sorted(categories.items()):
        pct = (stats["passed"] / stats["runs"] * 100) if stats["runs"] > 0 else 0
        avg_time = stats["time"] / stats["tests"] if stats["tests"] > 0 else 0
        
        # Emoji según precisión
        if pct == 100:
            emoji = "🎯"
        elif pct >= 70:
            emoji = "✅"
        elif pct >= 40:
            emoji = "⚠️"
        else:
            emoji = "❌"
        
        md += f"| {cat} | {stats['tests']} | {stats['passed']}/{stats['runs']} | {emoji} **{pct:.1f}%** | {avg_time:.0f}ms |\n"
    
    md += "\n---\n\n## 📋 Resultados Detallados\n\n"
    md += "**Leyenda:** 🎯 100% | ✅ ≥70% | ⚠️ ≥40% | ❌ <40%\n\n"
    md += "| Test | Categoría | Precisión | Pasaron | Tiempo | Tokens | T/s | Esperado |\n"
    md += "|------|-----------|-----------|---------|--------|--------|-----|----------|\n"
    
    # Agregar filas de resultados
    for r in results:
        # Emoji según precisión
        if r.accuracy == 100:
            emoji = "🎯"
        elif r.accuracy >= 70:
            emoji = "✅"
        elif r.accuracy >= 40:
            emoji = "⚠️"
        else:
            emoji = "❌"
        
        md += f"| {r.name} | {r.category} | {emoji} **{r.accuracy:.0f}%** | {r.passed}/{r.runs} | {r.avg_time_ms}ms | {r.avg_tokens} | {r.avg_tps} | `{r.expected}` |\n"
    
    md += "\n---\n\n## 📝 Muestras de Respuestas\n\n"
    
    # Agregar muestras de respuestas (solo las primeras 5)
    for i, r in enumerate(results[:5], 1):
        md += f"### {i}. {r.name} ({r.category})\n\n"
        md += f"**Esperado:** `{r.expected}`  \n"
        md += f"**Precisión:** {r.accuracy:.0f}%\n\n"
        
        if r.responses:
            md += "**Ejemplo de respuesta:**\n```\n"
            sample = r.responses[0][:200]
            if len(r.responses[0]) > 200:
                sample += "..."
            md += sample + "\n```\n\n"
    
    md += "---\n\n"
    md += f"*Generado por neuro-bitnet benchmark suite*  \n"
    md += f"*Cada test se ejecutó {runs} veces para medir precisión estadística*\n"
    
    return md

# =============================================================================
# Reporte Comparativo (LLM vs RAG)
# =============================================================================

def generate_comparison_markdown(results_llm: list, results_rag: list, url: str, runs: int) -> str:
    """Genera reporte comparando LLM directo vs RAG inteligente"""
    
    # Calcular métricas para LLM
    total_tests = len(results_llm)
    total_runs = total_tests * runs
    
    llm_passed = sum(r.passed for r in results_llm)
    llm_accuracy = (llm_passed / total_runs) * 100 if total_runs > 0 else 0
    llm_time = sum(r.avg_time_ms for r in results_llm) / len(results_llm) if results_llm else 0
    llm_tps = sum(r.avg_tps for r in results_llm) / len(results_llm) if results_llm else 0
    
    # Calcular métricas para RAG
    rag_passed = sum(r.passed for r in results_rag)
    rag_accuracy = (rag_passed / total_runs) * 100 if total_runs > 0 else 0
    rag_time = sum(r.avg_time_ms for r in results_rag) / len(results_rag) if results_rag else 0
    rag_tps = sum(r.avg_tps for r in results_rag) / len(results_rag) if results_rag else 0
    
    # Mejora
    accuracy_diff = rag_accuracy - llm_accuracy
    accuracy_emoji = "📈" if accuracy_diff > 0 else ("📉" if accuracy_diff < 0 else "➡️")
    
    md = f"""# 🧪 BitNet Benchmark Report - Comparación LLM vs RAG Inteligente

**Fecha:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}  
**Server LLM:** {url}  
**Server RAG:** {RAG_SERVER_URL}  
**Ejecuciones:** Cada test ejecutado {runs} veces

---

## 📊 Resumen Comparativo

| Métrica | 🔵 LLM Directo | 🟢 RAG Inteligente | Diferencia |
|---------|---------------|-------------------|------------|
| **Precisión Global** | **{llm_accuracy:.1f}%** | **{rag_accuracy:.1f}%** | {accuracy_emoji} **{accuracy_diff:+.1f}%** |
| Tests Pasados | {llm_passed}/{total_runs} | {rag_passed}/{total_runs} | {rag_passed - llm_passed:+d} |
| Tiempo Promedio | {llm_time:.0f}ms | {rag_time:.0f}ms | {rag_time - llm_time:+.0f}ms |
| Velocidad (t/s) | {llm_tps:.1f} | {rag_tps:.1f} | {rag_tps - llm_tps:+.1f} |

---

## 📊 Comparación por Categoría

| Categoría | 🔵 LLM | 🟢 RAG | Mejora | Estrategia RAG |
|-----------|--------|--------|--------|----------------|
"""
    
    # Agrupar por categoría
    categories_llm = {}
    categories_rag = {}
    
    for r in results_llm:
        if r.category not in categories_llm:
            categories_llm[r.category] = {"passed": 0, "runs": 0}
        categories_llm[r.category]["passed"] += r.passed
        categories_llm[r.category]["runs"] += r.runs
    
    for r in results_rag:
        if r.category not in categories_rag:
            categories_rag[r.category] = {"passed": 0, "runs": 0}
        categories_rag[r.category]["passed"] += r.passed
        categories_rag[r.category]["runs"] += r.runs
    
    # Estrategias esperadas por categoría
    expected_strategies = {
        "Chat": "RAG → Web (factual)",
        "Matemáticas": "LLM Directo",
        "Código": "LLM Directo",
        "Tools": "LLM Directo",
        "Razonamiento": "LLM Directo",
        "General": "LLM Directo (saludos)",
    }
    
    for cat in sorted(categories_llm.keys()):
        llm_stats = categories_llm.get(cat, {"passed": 0, "runs": 1})
        rag_stats = categories_rag.get(cat, {"passed": 0, "runs": 1})
        
        llm_pct = (llm_stats["passed"] / llm_stats["runs"] * 100) if llm_stats["runs"] > 0 else 0
        rag_pct = (rag_stats["passed"] / rag_stats["runs"] * 100) if rag_stats["runs"] > 0 else 0
        diff = rag_pct - llm_pct
        
        if diff > 5:
            emoji = "🎯"
        elif diff > 0:
            emoji = "✅"
        elif diff == 0:
            emoji = "➡️"
        else:
            emoji = "⚠️"
        
        strategy = expected_strategies.get(cat, "Auto")
        md += f"| {cat} | {llm_pct:.0f}% | {rag_pct:.0f}% | {emoji} {diff:+.0f}% | {strategy} |\n"
    
    md += "\n---\n\n## 📋 Resultados Detallados por Test\n\n"
    md += "| Test | Categoría | 🔵 LLM | 🟢 RAG | Mejora |\n"
    md += "|------|-----------|--------|--------|--------|\n"
    
    for i, r_llm in enumerate(results_llm):
        r_rag = results_rag[i] if i < len(results_rag) else None
        
        llm_acc = r_llm.accuracy
        rag_acc = r_rag.accuracy if r_rag else 0
        diff = rag_acc - llm_acc
        
        if diff > 10:
            emoji = "🎯"
        elif diff > 0:
            emoji = "✅"
        elif diff == 0:
            emoji = "➡️"
        else:
            emoji = "⚠️"
        
        md += f"| {r_llm.name} | {r_llm.category} | {llm_acc:.0f}% | {rag_acc:.0f}% | {emoji} {diff:+.0f}% |\n"
    
    # Tests con mayor mejora
    improvements = []
    for i, r_llm in enumerate(results_llm):
        if i < len(results_rag):
            r_rag = results_rag[i]
            diff = r_rag.accuracy - r_llm.accuracy
            if diff > 0:
                improvements.append((r_llm.name, r_llm.category, r_llm.accuracy, r_rag.accuracy, diff))
    
    if improvements:
        improvements.sort(key=lambda x: x[4], reverse=True)
        md += "\n---\n\n## 🎯 Tests con Mayor Mejora usando RAG\n\n"
        md += "| Test | Categoría | LLM → RAG | Mejora |\n"
        md += "|------|-----------|-----------|--------|\n"
        for name, cat, llm_acc, rag_acc, diff in improvements[:5]:
            md += f"| {name} | {cat} | {llm_acc:.0f}% → {rag_acc:.0f}% | **+{diff:.0f}%** |\n"
    
    # Tests con degradación
    degradations = []
    for i, r_llm in enumerate(results_llm):
        if i < len(results_rag):
            r_rag = results_rag[i]
            diff = r_rag.accuracy - r_llm.accuracy
            if diff < 0:
                degradations.append((r_llm.name, r_llm.category, r_llm.accuracy, r_rag.accuracy, diff))
    
    if degradations:
        degradations.sort(key=lambda x: x[4])
        md += "\n---\n\n## ⚠️ Tests donde RAG fue Peor\n\n"
        md += "| Test | Categoría | LLM → RAG | Diferencia |\n"
        md += "|------|-----------|-----------|------------|\n"
        for name, cat, llm_acc, rag_acc, diff in degradations:
            md += f"| {name} | {cat} | {llm_acc:.0f}% → {rag_acc:.0f}% | **{diff:.0f}%** |\n"
    
    # Muestras de respuestas comparativas
    md += "\n---\n\n## 📝 Ejemplos de Respuestas Comparativas\n\n"
    
    # Mostrar tests donde hubo diferencia significativa
    interesting_tests = []
    for i, r_llm in enumerate(results_llm):
        if i < len(results_rag):
            r_rag = results_rag[i]
            diff = abs(r_rag.accuracy - r_llm.accuracy)
            if diff >= 30:  # Diferencia significativa
                interesting_tests.append((r_llm, r_rag, diff))
    
    interesting_tests.sort(key=lambda x: x[2], reverse=True)
    
    for r_llm, r_rag, diff in interesting_tests[:3]:
        md += f"### {r_llm.name} ({r_llm.category})\n\n"
        md += f"**Esperado:** `{r_llm.expected}`\n\n"
        md += f"| Modo | Precisión | Respuesta |\n"
        md += f"|------|-----------|----------|\n"
        
        llm_sample = r_llm.responses[0][:100] if r_llm.responses else "N/A"
        rag_sample = r_rag.responses[0][:100] if r_rag.responses else "N/A"
        
        md += f"| 🔵 LLM | {r_llm.accuracy:.0f}% | `{llm_sample}...` |\n"
        md += f"| 🟢 RAG | {r_rag.accuracy:.0f}% | `{rag_sample}...` |\n\n"
    
    # Conclusiones
    md += "\n---\n\n## 📈 Conclusiones\n\n"
    
    if accuracy_diff > 5:
        md += f"✅ **El RAG Inteligente mejora la precisión en {accuracy_diff:.1f}%**\n\n"
        md += "El sistema RAG clasifica las consultas y usa la estrategia óptima:\n"
        md += "- **Consultas factuales** (capitales, personas, historia): Busca en RAG/Web\n"
        md += "- **Matemáticas, código, razonamiento**: Usa LLM directo (más rápido)\n"
    elif accuracy_diff < -5:
        md += f"⚠️ **El RAG Inteligente tuvo menor precisión ({accuracy_diff:.1f}%)**\n\n"
        md += "Posibles causas:\n"
        md += "- Información desactualizada en la web\n"
        md += "- Contexto RAG introduciendo ruido\n"
    else:
        md += f"➡️ **Precisión similar entre ambos modos ({accuracy_diff:+.1f}%)**\n\n"
        md += "El RAG añade valor en consultas factuales sin degradar las demás.\n"
    
    time_diff = rag_time - llm_time
    if time_diff > 500:
        md += f"\n⏱️ **El RAG es {time_diff:.0f}ms más lento en promedio** (debido a búsquedas web)\n"
    elif time_diff < -100:
        md += f"\n⚡ **El RAG es {-time_diff:.0f}ms más rápido** (caché de embeddings)\n"
    
    md += "\n---\n\n"
    md += f"*Generado por neuro-bitnet benchmark suite*  \n"
    md += f"*Comparación: LLM directo vs RAG Server Inteligente*  \n"
    md += f"*Cada test se ejecutó {runs} veces*\n"
    
    return md

# =============================================================================
# Main
# =============================================================================

def main():
    parser = argparse.ArgumentParser(description="BitNet Report Generator con análisis de precisión")
    parser.add_argument("--url", default=DEFAULT_URL)
    parser.add_argument("--output", "-o", default="benchmark_report.md")
    parser.add_argument("--runs", "-r", type=int, default=DEFAULT_RUNS,
                        help=f"Número de veces que se ejecuta cada test (default: {DEFAULT_RUNS})")
    parser.add_argument("--rag", action="store_true",
                        help="Usar sistema RAG con auto-learn (búsqueda web cuando no sabe)")
    parser.add_argument("--smart-rag", action="store_true",
                        help="Usar RAG Server inteligente (clasificación automática)")
    parser.add_argument("--compare", action="store_true",
                        help="Comparar LLM directo vs RAG inteligente (genera ambos)")
    parser.add_argument("--rag-url", default=None,
                        help=f"URL del RAG Server (default: {RAG_SERVER_URL})")
    parser.add_argument("--category", "-c", type=str, default=None,
                        help="Filtrar por categoría: Chat, Matemáticas, Código, Tools, Razonamiento, General")
    args = parser.parse_args()
    
    # Actualizar URL del RAG server si se especifica
    rag_server_url = args.rag_url if args.rag_url else RAG_SERVER_URL
    
    print(f"\n🧪 BitNet Benchmark - Análisis de Precisión")
    print(f"🔗 URL LLM: {args.url}")
    if args.compare or args.smart_rag:
        print(f"🔗 URL RAG: {rag_server_url}")
    print(f"🔄 Runs por test: {args.runs}")
    
    if args.compare:
        print(f"📊 Modo: Comparación LLM vs RAG Inteligente")
    elif args.smart_rag:
        print(f"🧠 Modo: RAG Server inteligente")
    elif args.rag:
        print(f"🧠 Modo: RAG con auto-learn")
    else:
        print(f"💬 Modo: LLM directo")
    
    if args.category:
        print(f"📂 Categoría: {args.category}")
    print("=" * 60)
    
    # Verificar servidor LLM
    try:
        r = requests.get(f"{args.url}/health", timeout=5)
        if r.status_code != 200:
            raise Exception("Health check failed")
        print("✅ Servidor LLM disponible")
    except:
        print(f"❌ No se puede conectar al LLM en {args.url}")
        exit(1)
    
    # Verificar RAG Server si es necesario
    if args.compare or args.smart_rag:
        try:
            r = requests.get(f"{rag_server_url}/health", timeout=5)
            if r.status_code != 200:
                raise Exception("RAG Health check failed")
            print("✅ RAG Server disponible")
        except:
            print(f"❌ No se puede conectar al RAG Server en {rag_server_url}")
            print(f"   Inicia el servidor: python scripts/rag_server.py")
            exit(1)
    
    # Modo comparación: ejecutar ambos
    if args.compare:
        print("\n" + "=" * 60)
        print("🔵 FASE 1: Tests con LLM Directo")
        print("=" * 60)
        results_llm = run_tests(args.url, args.runs, use_rag=False, 
                                use_smart_rag=False, category_filter=args.category)
        
        print("\n" + "=" * 60)
        print("🟢 FASE 2: Tests con RAG Inteligente")
        print("=" * 60)
        results_rag = run_tests(args.url, args.runs, use_rag=False,
                                use_smart_rag=True, category_filter=args.category)
        
        if not results_llm or not results_rag:
            exit(1)
        
        # Generar reporte comparativo
        markdown = generate_comparison_markdown(results_llm, results_rag, args.url, args.runs)
        
        # Calcular métricas finales
        llm_passed = sum(r.passed for r in results_llm)
        rag_passed = sum(r.passed for r in results_rag)
        total_runs = sum(r.runs for r in results_llm)
        
        llm_accuracy = (llm_passed / total_runs) * 100 if total_runs > 0 else 0
        rag_accuracy = (rag_passed / total_runs) * 100 if total_runs > 0 else 0
        diff = rag_accuracy - llm_accuracy
        
        print("\n" + "=" * 60)
        print("📊 COMPARACIÓN FINAL")
        print("=" * 60)
        print(f"🔵 LLM Directo:      {llm_accuracy:.1f}%")
        print(f"🟢 RAG Inteligente:  {rag_accuracy:.1f}%")
        print(f"📈 Diferencia:       {diff:+.1f}%")
        
    else:
        # Modo simple: un solo tipo de test
        results = run_tests(args.url, args.runs, use_rag=args.rag,
                           use_smart_rag=args.smart_rag, category_filter=args.category)
        
        if not results:
            exit(1)
        
        # Generar reporte
        markdown = generate_markdown(results, args.url, args.runs)
        
        # Resumen
        total_passed = sum(r.passed for r in results)
        total_runs = sum(r.runs for r in results)
        overall_accuracy = (total_passed / total_runs) * 100 if total_runs > 0 else 0
        perfect = sum(1 for r in results if r.accuracy == 100)
        
        print("\n" + "=" * 60)
        print(f"📊 PRECISIÓN GLOBAL: {overall_accuracy:.1f}%")
        print(f"🎯 Tests con 100%: {perfect}/{len(results)}")
    
    # Guardar reporte
    with open(args.output, 'w', encoding='utf-8') as f:
        f.write(markdown)
    
    print(f"📄 Reporte generado: {args.output}")
    print("=" * 60)

if __name__ == "__main__":
    main()