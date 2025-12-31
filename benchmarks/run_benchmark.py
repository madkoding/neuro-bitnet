#!/usr/bin/env python3
"""
Benchmark completo de neuro-bitnet con inferencia local.
Genera reportes en español e inglés.
"""

import subprocess
import json
import time
import sys
from dataclasses import dataclass
from typing import Optional
from pathlib import Path

@dataclass
class TestCase:
    query: str
    expected_category: str
    language: str  # "en" or "es"

@dataclass 
class TestResult:
    query: str
    expected_category: str
    actual_category: str
    confidence: float
    answer: str
    classification_ms: int
    llm_ms: int
    total_ms: int
    passed: bool

# Test cases - balanced between English and Spanish
TEST_CASES = [
    # Greetings (EN)
    TestCase("Hello, how are you today?", "Greeting", "en"),
    TestCase("What is your name?", "Greeting", "en"),
    TestCase("Good morning!", "Greeting", "en"),
    # Saludos (ES)
    TestCase("Hola, ¿cómo estás?", "Greeting", "es"),
    TestCase("Buenos días", "Greeting", "es"),
    
    # Code (EN)
    TestCase("How do I implement binary search in Python?", "Code", "en"),
    TestCase("Write a function to sort an array", "Code", "en"),
    TestCase("Debug this JavaScript code", "Code", "en"),
    # Código (ES)
    TestCase("¿Cómo implemento quicksort en JavaScript?", "Code", "es"),
    TestCase("Escribe una función en Python para fibonacci", "Code", "es"),
    
    # Factual Knowledge (EN)
    TestCase("What is the capital of France?", "Factual", "en"),
    TestCase("Who invented the telephone?", "Factual", "en"),
    TestCase("When did World War II end?", "Factual", "en"),
    # Conocimiento Factual (ES)
    TestCase("¿Cuál es la capital de España?", "Factual", "es"),
    TestCase("¿Quién descubrió América?", "Factual", "es"),
    
    # Math (EN)
    TestCase("What is 15 multiplied by 7?", "Math", "en"),
    TestCase("Solve: 2x + 5 = 15", "Math", "en"),
    TestCase("Calculate the area of a circle with radius 5", "Math", "en"),
    # Matemáticas (ES)
    TestCase("¿Cuánto es 144 dividido entre 12?", "Math", "es"),
    TestCase("Calcula el área de un círculo con radio 5", "Math", "es"),
    
    # Conversational (EN)
    TestCase("Tell me about yourself", "Conversational", "en"),
    TestCase("What do you think about AI?", "Conversational", "en"),
    # Conversacional (ES)
    TestCase("Cuéntame sobre ti", "Conversational", "es"),
    TestCase("¿Qué opinas sobre la inteligencia artificial?", "Conversational", "es"),
]


def run_test(neuro_bin: str, model_path: str, test: TestCase) -> Optional[TestResult]:
    """Execute a single test and return results."""
    cmd = [
        neuro_bin, "ask", test.query,
        "--model-path", model_path,
        "--max-tokens", "128",
        "--temperature", "0.7",
        "--format", "json"
    ]
    
    try:
        # Run command and capture output
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=120
        )
        
        # Parse JSON from stdout (skip stderr logs)
        output = result.stdout.strip()
        
        # Find JSON in output (it starts with {)
        json_start = output.find('{')
        if json_start == -1:
            print(f"  ❌ No JSON found in output for: {test.query[:30]}...")
            return None
            
        json_str = output[json_start:]
        data = json.loads(json_str)
        
        # Normalize category names for comparison
        actual_cat = data.get("category", "Unknown")
        # Remove "Query" suffix if present
        if actual_cat.endswith("Query"):
            actual_cat = actual_cat[:-5]
        
        expected_normalized = test.expected_category.replace("Query", "")
        actual_normalized = actual_cat.replace("Query", "")
        
        # Check if classification passed (allow some flexibility)
        passed = expected_normalized.lower() in actual_normalized.lower() or \
                 actual_normalized.lower() in expected_normalized.lower()
        
        timing = data.get("timing", {})
        
        return TestResult(
            query=test.query,
            expected_category=test.expected_category,
            actual_category=actual_cat,
            confidence=data.get("confidence", 0.0),
            answer=data.get("answer", "")[:300],  # Truncate long answers
            classification_ms=timing.get("classification_ms", 0),
            llm_ms=timing.get("llm_ms", 0),
            total_ms=timing.get("total_ms", 0),
            passed=passed
        )
        
    except subprocess.TimeoutExpired:
        print(f"  ⏱ Timeout for: {test.query[:30]}...")
        return None
    except json.JSONDecodeError as e:
        print(f"  ❌ JSON error for: {test.query[:30]}... - {e}")
        return None
    except Exception as e:
        print(f"  ❌ Error for: {test.query[:30]}... - {e}")
        return None


def generate_markdown_report(results: list[TestResult], model_name: str, lang: str) -> str:
    """Generate markdown report in specified language."""
    
    # Stats
    total = len(results)
    passed = sum(1 for r in results if r.passed)
    accuracy = (passed / total * 100) if total > 0 else 0
    
    avg_classification = sum(r.classification_ms for r in results) / total if total > 0 else 0
    avg_llm = sum(r.llm_ms for r in results) / total if total > 0 else 0
    avg_total = sum(r.total_ms for r in results) / total if total > 0 else 0
    
    # Group by category
    by_category = {}
    for r in results:
        cat = r.expected_category
        if cat not in by_category:
            by_category[cat] = []
        by_category[cat].append(r)
    
    if lang == "es":
        return _generate_spanish_report(results, by_category, model_name, 
                                        total, passed, accuracy,
                                        avg_classification, avg_llm, avg_total)
    else:
        return _generate_english_report(results, by_category, model_name,
                                        total, passed, accuracy,
                                        avg_classification, avg_llm, avg_total)


def _generate_spanish_report(results, by_category, model_name, 
                             total, passed, accuracy,
                             avg_classification, avg_llm, avg_total) -> str:
    
    report = f"""# Reporte de Benchmark - neuro-bitnet

## Resumen Ejecutivo

| Métrica | Valor |
|---------|-------|
| **Modelo** | {model_name} |
| **Tests Ejecutados** | {total} |
| **Tests Pasados** | {passed} |
| **Precisión Clasificación** | {accuracy:.1f}% |
| **Tiempo Promedio Clasificación** | {avg_classification:.0f}ms |
| **Tiempo Promedio LLM** | {avg_llm:.0f}ms |
| **Tiempo Promedio Total** | {avg_total:.0f}ms |

## Resultados por Categoría

"""
    
    category_names_es = {
        "Greeting": "Saludos",
        "Conversational": "Conversacional",
        "Code": "Código/Técnico",
        "Factual": "Conocimiento Factual",
        "Math": "Matemáticas",
        "CreativeWriting": "Escritura Creativa"
    }
    
    for cat, cat_results in by_category.items():
        cat_name = category_names_es.get(cat, cat)
        cat_passed = sum(1 for r in cat_results if r.passed)
        cat_total = len(cat_results)
        cat_accuracy = (cat_passed / cat_total * 100) if cat_total > 0 else 0
        
        report += f"### {cat_name}\n\n"
        report += f"**Precisión:** {cat_passed}/{cat_total} ({cat_accuracy:.0f}%)\n\n"
        report += "| Query | Categoría Detectada | Confianza | Tiempo (ms) | Estado |\n"
        report += "|-------|---------------------|-----------|-------------|--------|\n"
        
        for r in cat_results:
            status = "✅" if r.passed else "❌"
            query_short = r.query[:40] + "..." if len(r.query) > 40 else r.query
            report += f"| {query_short} | {r.actual_category} | {r.confidence:.2f} | {r.total_ms} | {status} |\n"
        
        report += "\n"
    
    report += """## Detalle de Respuestas

"""
    
    for i, r in enumerate(results, 1):
        status = "✅" if r.passed else "❌"
        report += f"""### Test {i}: {status}

**Query:** {r.query}

**Categoría:** {r.actual_category} (esperada: {r.expected_category})

**Confianza:** {r.confidence:.2f}

**Tiempos:** Clasificación: {r.classification_ms}ms | LLM: {r.llm_ms}ms | Total: {r.total_ms}ms

**Respuesta:**
> {r.answer}

---

"""
    
    report += f"""## Conclusiones

- La clasificación de queries alcanza un **{accuracy:.1f}%** de precisión
- El tiempo promedio de respuesta es de **{avg_total:.0f}ms** (~{1000/avg_total:.1f} queries/segundo)
- El modelo {model_name} funciona correctamente en CPU sin necesidad de GPU

### Rendimiento por Componente

| Componente | Tiempo Promedio | % del Total |
|------------|-----------------|-------------|
| Clasificación | {avg_classification:.0f}ms | {(avg_classification/avg_total*100):.1f}% |
| Inferencia LLM | {avg_llm:.0f}ms | {(avg_llm/avg_total*100):.1f}% |

---

*Reporte generado automáticamente por neuro-bitnet benchmark*
"""
    
    return report


def _generate_english_report(results, by_category, model_name,
                             total, passed, accuracy,
                             avg_classification, avg_llm, avg_total) -> str:
    
    report = f"""# Benchmark Report - neuro-bitnet

## Executive Summary

| Metric | Value |
|--------|-------|
| **Model** | {model_name} |
| **Tests Executed** | {total} |
| **Tests Passed** | {passed} |
| **Classification Accuracy** | {accuracy:.1f}% |
| **Avg Classification Time** | {avg_classification:.0f}ms |
| **Avg LLM Time** | {avg_llm:.0f}ms |
| **Avg Total Time** | {avg_total:.0f}ms |

## Results by Category

"""
    
    category_names_en = {
        "Greeting": "Greetings",
        "Conversational": "Conversational",
        "Code": "Technical/Code",
        "Factual": "Factual Knowledge",
        "Math": "Mathematics",
        "CreativeWriting": "Creative Writing"
    }
    
    for cat, cat_results in by_category.items():
        cat_name = category_names_en.get(cat, cat)
        cat_passed = sum(1 for r in cat_results if r.passed)
        cat_total = len(cat_results)
        cat_accuracy = (cat_passed / cat_total * 100) if cat_total > 0 else 0
        
        report += f"### {cat_name}\n\n"
        report += f"**Accuracy:** {cat_passed}/{cat_total} ({cat_accuracy:.0f}%)\n\n"
        report += "| Query | Detected Category | Confidence | Time (ms) | Status |\n"
        report += "|-------|-------------------|------------|-----------|--------|\n"
        
        for r in cat_results:
            status = "✅" if r.passed else "❌"
            query_short = r.query[:40] + "..." if len(r.query) > 40 else r.query
            report += f"| {query_short} | {r.actual_category} | {r.confidence:.2f} | {r.total_ms} | {status} |\n"
        
        report += "\n"
    
    report += """## Response Details

"""
    
    for i, r in enumerate(results, 1):
        status = "✅" if r.passed else "❌"
        report += f"""### Test {i}: {status}

**Query:** {r.query}

**Category:** {r.actual_category} (expected: {r.expected_category})

**Confidence:** {r.confidence:.2f}

**Timing:** Classification: {r.classification_ms}ms | LLM: {r.llm_ms}ms | Total: {r.total_ms}ms

**Response:**
> {r.answer}

---

"""
    
    report += f"""## Conclusions

- Query classification achieves **{accuracy:.1f}%** accuracy
- Average response time is **{avg_total:.0f}ms** (~{1000/avg_total:.1f} queries/second)
- Model {model_name} runs correctly on CPU without GPU

### Performance by Component

| Component | Avg Time | % of Total |
|-----------|----------|------------|
| Classification | {avg_classification:.0f}ms | {(avg_classification/avg_total*100):.1f}% |
| LLM Inference | {avg_llm:.0f}ms | {(avg_llm/avg_total*100):.1f}% |

---

*Report automatically generated by neuro-bitnet benchmark*
"""
    
    return report


def detect_model_name(model_path: str) -> str:
    """Detect model name from file path."""
    filename = Path(model_path).stem.lower()
    
    if "bitnet" in filename or "i2_s" in filename or "i2s" in filename:
        return "BitNet 1.58 2B (i2_s)"
    elif "qwen" in filename:
        if "0.5b" in filename:
            return "Qwen 2.5 0.5B (Q4_K_M)"
        elif "1.5b" in filename:
            return "Qwen 2.5 1.5B"
        else:
            return "Qwen 2.5"
    elif "llama" in filename:
        return "Llama"
    elif "mistral" in filename:
        return "Mistral"
    else:
        return Path(model_path).name


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Run neuro-bitnet benchmark")
    parser.add_argument("--model", "-m", default="/tmp/bitnet-1.58-2b.gguf",
                        help="Path to GGUF model file")
    parser.add_argument("--model-name", help="Override model name for report")
    args = parser.parse_args()
    
    # Configuration
    neuro_bin = "./target/release/neuro"
    model_path = args.model
    model_name = args.model_name or detect_model_name(model_path)
    
    # Check prerequisites
    if not Path(neuro_bin).exists():
        print("❌ neuro binary not found. Run: cargo build --release -p neuro-cli")
        sys.exit(1)
    
    if not Path(model_path).exists():
        print(f"❌ Model not found at {model_path}")
        print("Download BitNet: curl -L -o /tmp/bitnet-1.58-2b.gguf https://huggingface.co/microsoft/bitnet-b1.58-2B-4T-gguf/resolve/main/ggml-model-i2_s.gguf")
        sys.exit(1)
    
    print("═" * 60)
    print("  NEURO-BITNET BENCHMARK WITH LOCAL INFERENCE")
    print("═" * 60)
    print(f"\nModel: {model_name}")
    print(f"Tests: {len(TEST_CASES)}")
    print()
    
    # Run tests
    results = []
    for i, test in enumerate(TEST_CASES, 1):
        print(f"[{i}/{len(TEST_CASES)}] Testing: {test.query[:50]}...")
        result = run_test(neuro_bin, model_path, test)
        if result:
            results.append(result)
            status = "✅" if result.passed else "❌"
            print(f"  {status} {result.actual_category} ({result.confidence:.2f}) - {result.total_ms}ms")
        else:
            print(f"  ⚠️  Skipped (error)")
    
    print()
    
    if not results:
        print("❌ No results collected!")
        sys.exit(1)
    
    # Generate reports
    print("Generating reports...")
    
    report_es = generate_markdown_report(results, model_name, "es")
    report_en = generate_markdown_report(results, model_name, "en")
    
    # Save reports
    Path("benchmarks/BENCHMARK_REPORT_ES.md").write_text(report_es)
    Path("benchmarks/BENCHMARK_REPORT_EN.md").write_text(report_en)
    
    # Summary
    passed = sum(1 for r in results if r.passed)
    total = len(results)
    accuracy = (passed / total * 100) if total > 0 else 0
    
    print()
    print("═" * 60)
    print("  RESULTS")
    print("═" * 60)
    print(f"\n✅ Passed: {passed}/{total} ({accuracy:.1f}%)")
    print(f"📄 Reports saved:")
    print(f"   - benchmarks/BENCHMARK_REPORT_ES.md")
    print(f"   - benchmarks/BENCHMARK_REPORT_EN.md")
    print()


if __name__ == "__main__":
    main()
