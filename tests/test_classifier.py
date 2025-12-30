#!/usr/bin/env python3
"""
Test del clasificador de consultas
===================================
Verifica que el clasificador categorice correctamente las consultas.
"""

import sys
sys.path.insert(0, '/home/madkoding/proyectos/neuro-bitnet/scripts')

from rag_server import QueryClassifier, QueryCategory, QueryStrategy

def test_classifier():
    classifier = QueryClassifier()
    
    tests = [
        # (pregunta, categoría esperada, estrategia esperada)
        
        # Matemáticas
        ("¿Cuánto es 25+17?", QueryCategory.MATH, QueryStrategy.LLM_DIRECT),
        ("Calcula 100/4", QueryCategory.MATH, QueryStrategy.LLM_DIRECT),
        ("12*11", QueryCategory.MATH, QueryStrategy.LLM_DIRECT),
        ("¿Cuál es la raíz cuadrada de 144?", QueryCategory.MATH, QueryStrategy.LLM_DIRECT),
        
        # Código
        ("Escribe una función en Python", QueryCategory.CODE, QueryStrategy.LLM_DIRECT),
        ("Crea una clase en JavaScript", QueryCategory.CODE, QueryStrategy.LLM_DIRECT),
        ("¿Cómo hago un bucle for?", QueryCategory.CODE, QueryStrategy.LLM_DIRECT),
        ("print hola mundo", QueryCategory.CODE, QueryStrategy.LLM_DIRECT),
        
        # Razonamiento
        ("Si todos los perros son mamíferos, entonces...", QueryCategory.REASONING, QueryStrategy.LLM_DIRECT),
        ("¿Qué sigue en la secuencia 2, 4, 6, 8?", QueryCategory.REASONING, QueryStrategy.LLM_DIRECT),
        ("Por lo tanto, lógicamente...", QueryCategory.REASONING, QueryStrategy.LLM_DIRECT),
        
        # Tools
        ("¿Cuál es el clima en Madrid?", QueryCategory.TOOLS, QueryStrategy.LLM_DIRECT),
        ("Traduce esto al inglés", QueryCategory.TOOLS, QueryStrategy.LLM_DIRECT),
        
        # Saludos
        ("Hola, ¿cómo estás?", QueryCategory.GREETING, QueryStrategy.LLM_DIRECT),
        ("Buenos días", QueryCategory.GREETING, QueryStrategy.LLM_DIRECT),
        ("Adiós, hasta luego", QueryCategory.GREETING, QueryStrategy.LLM_DIRECT),
        
        # Factual (requiere RAG)
        ("¿Cuál es la capital de Francia?", QueryCategory.FACTUAL, QueryStrategy.RAG_THEN_WEB),
        ("¿Quién fue Albert Einstein?", QueryCategory.FACTUAL, QueryStrategy.RAG_THEN_WEB),
        ("¿Cuándo nació Leonardo da Vinci?", QueryCategory.FACTUAL, QueryStrategy.RAG_THEN_WEB),
        ("¿Qué es la fotosíntesis?", QueryCategory.FACTUAL, QueryStrategy.RAG_THEN_WEB),
        ("Historia de Roma", QueryCategory.FACTUAL, QueryStrategy.RAG_THEN_WEB),
        ("¿Quién inventó el teléfono?", QueryCategory.FACTUAL, QueryStrategy.RAG_THEN_WEB),
    ]
    
    passed = 0
    failed = 0
    
    print("=" * 70)
    print("🧪 Test del Clasificador de Consultas")
    print("=" * 70)
    
    for question, expected_cat, expected_strat in tests:
        result = classifier.classify(question)
        
        cat_ok = result.category == expected_cat
        strat_ok = result.strategy == expected_strat
        
        if cat_ok and strat_ok:
            status = "✅"
            passed += 1
        else:
            status = "❌"
            failed += 1
        
        print(f"\n{status} \"{question[:50]}...\"" if len(question) > 50 else f"\n{status} \"{question}\"")
        
        if not cat_ok:
            print(f"   Categoría: {result.category.value} (esperado: {expected_cat.value})")
        if not strat_ok:
            print(f"   Estrategia: {result.strategy.value} (esperado: {expected_strat.value})")
        
        if cat_ok and strat_ok:
            print(f"   {result.category.value} → {result.strategy.value} ({result.confidence:.0%})")
    
    print("\n" + "=" * 70)
    print(f"📊 Resultados: {passed}/{passed+failed} tests pasados ({100*passed/(passed+failed):.0f}%)")
    print("=" * 70)
    
    return failed == 0


if __name__ == "__main__":
    success = test_classifier()
    sys.exit(0 if success else 1)
