"""
Unit Tests for Classifier Module
=================================

Tests para QueryClassifier, categorización y estrategias.
"""

import pytest
from src.rag.classifier import (
    QueryClassifier, 
    QueryCategory, 
    QueryStrategy,
    ClassificationResult
)


class TestQueryCategory:
    """Tests para el enum QueryCategory."""
    
    def test_all_categories_exist(self):
        """Test que todas las categorías están definidas."""
        expected = {"math", "code", "reasoning", "tools", "greeting", "factual", "conversational"}
        actual = {c.value for c in QueryCategory}
        
        assert expected == actual
    
    def test_category_values(self):
        """Test valores específicos."""
        assert QueryCategory.MATH.value == "math"
        assert QueryCategory.CODE.value == "code"
        assert QueryCategory.FACTUAL.value == "factual"


class TestQueryStrategy:
    """Tests para el enum QueryStrategy."""
    
    def test_all_strategies_exist(self):
        """Test que todas las estrategias están definidas."""
        strategies = {s.value for s in QueryStrategy}
        
        assert "llm_direct" in strategies
        assert "rag_local" in strategies
        assert "web_search" in strategies
        assert "rag_then_web" in strategies


class TestClassificationResult:
    """Tests para ClassificationResult."""
    
    def test_create_result(self):
        """Test crear resultado de clasificación."""
        result = ClassificationResult(
            category=QueryCategory.MATH,
            strategy=QueryStrategy.LLM_DIRECT,
            confidence=0.95,
            reasons=["cuanto es", "\\d+\\s*\\+"]
        )
        
        assert result.category == QueryCategory.MATH
        assert result.strategy == QueryStrategy.LLM_DIRECT
        assert result.confidence == 0.95
        assert len(result.reasons) == 2
    
    def test_result_to_dict(self):
        """Test serialización."""
        result = ClassificationResult(
            category=QueryCategory.CODE,
            strategy=QueryStrategy.RAG_LOCAL,
            confidence=0.85,
            reasons=["código detectado"]
        )
        
        d = result.to_dict()
        
        assert d["category"] == "code"
        assert d["strategy"] == "rag_local"
        assert d["confidence"] == 0.85
        assert "reasons" in d


class TestQueryClassifier:
    """Tests para QueryClassifier."""
    
    @pytest.fixture
    def classifier(self):
        """Crear instancia del clasificador."""
        return QueryClassifier()
    
    # === Tests de Matemáticas ===
    
    @pytest.mark.parametrize("query", [
        "cuánto es 5 + 3",
        "resultado de 10 * 5",
        "5+3",
        "raíz cuadrada de 16",
        "cuánto es el 15% de 200",
    ])
    def test_classify_math_queries(self, classifier, query):
        """Test que consultas matemáticas se clasifican correctamente."""
        result = classifier.classify(query)
        
        assert result.category == QueryCategory.MATH, f"Query '{query}' should be MATH"
        assert result.strategy == QueryStrategy.LLM_DIRECT
    
    # === Tests de Código ===
    
    @pytest.mark.parametrize("query", [
        "escribe una función en python",
        "crea una clase en Java",
        "código para ordenar una lista",
        "cómo implementa un bucle for",
        "cómo usar variables en javascript",
        "escribe un script con un loop",
        "función que ordene una lista",
    ])
    def test_classify_code_queries(self, classifier, query):
        """Test que consultas de código se clasifican correctamente."""
        result = classifier.classify(query)
        
        assert result.category == QueryCategory.CODE, f"Query '{query}' should be CODE"
        assert result.strategy == QueryStrategy.RAG_LOCAL
    
    # === Tests de Razonamiento ===
    
    @pytest.mark.parametrize("query", [
        "si todos los perros son animales entonces",
        "qué sigue en la secuencia 1, 2, 3",
        "lógicamente esto implica que",
        "verdadero o falso",
        "analiza este patrón",
    ])
    def test_classify_reasoning_queries(self, classifier, query):
        """Test que consultas de razonamiento se clasifican correctamente."""
        result = classifier.classify(query)
        
        assert result.category == QueryCategory.REASONING, f"Query '{query}' should be REASONING"
    
    # === Tests de Herramientas ===
    
    @pytest.mark.parametrize("query", [
        "clima en Madrid",
        "traduce esto al inglés",
        "buscar información sobre IA",
        "envía un mensaje a Juan",
        "crear un recordatorio",
        "calcula 25*4",
        "suma 10 y 5",
    ])
    def test_classify_tools_queries(self, classifier, query):
        """Test que consultas de herramientas se clasifican correctamente."""
        result = classifier.classify(query)
        
        assert result.category == QueryCategory.TOOLS, f"Query '{query}' should be TOOLS"
        assert result.strategy == QueryStrategy.LLM_DIRECT
    
    # === Tests de Saludos ===
    
    @pytest.mark.parametrize("query", [
        "hola",
        "buenos días",
        "buenas tardes",
        "hey",
        "saludos",
        "qué tal",
        "gracias",
        "adiós",
    ])
    def test_classify_greeting_queries(self, classifier, query):
        """Test que saludos se clasifican correctamente."""
        result = classifier.classify(query)
        
        assert result.category == QueryCategory.GREETING, f"Query '{query}' should be GREETING"
        assert result.strategy == QueryStrategy.LLM_DIRECT
    
    # === Tests de Factuales ===
    
    @pytest.mark.parametrize("query", [
        "quién fue Albert Einstein",
        "qué es la fotosíntesis",
        "capital de Francia",
        "cuándo nació Mozart",
        "dónde está la torre Eiffel",
        "historia de Roma",
        "inventor del teléfono",
    ])
    def test_classify_factual_queries(self, classifier, query):
        """Test que consultas factuales se clasifican correctamente."""
        result = classifier.classify(query)
        
        assert result.category == QueryCategory.FACTUAL, f"Query '{query}' should be FACTUAL"
        assert result.strategy == QueryStrategy.RAG_THEN_WEB
    
    # === Tests de Confianza ===
    
    def test_high_confidence_clear_pattern(self, classifier):
        """Test que patrones claros tienen alta confianza."""
        result = classifier.classify("5+3")
        
        assert result.confidence >= 0.5
    
    def test_lower_confidence_ambiguous(self, classifier):
        """Test que consultas ambiguas tienen menor confianza."""
        result = classifier.classify("esto es interesante")
        
        # Conversational queries should have lower confidence
        assert result.category == QueryCategory.CONVERSATIONAL
        assert result.confidence <= 0.6
    
    # === Tests de Razones ===
    
    def test_reasons_returned(self, classifier):
        """Test que se devuelven las razones de clasificación."""
        result = classifier.classify("quién fue Einstein")
        
        assert len(result.reasons) > 0
    
    # === Tests de Estrategia ===
    
    def test_factual_uses_rag_then_web(self, classifier):
        """Test factual usa RAG then Web."""
        result = classifier.classify("quién inventó el teléfono")
        
        assert result.strategy == QueryStrategy.RAG_THEN_WEB
    
    def test_code_uses_rag_local(self, classifier):
        """Test código usa RAG local."""
        result = classifier.classify("escribe un código en python")
        
        assert result.strategy == QueryStrategy.RAG_LOCAL
    
    def test_greeting_uses_direct(self, classifier):
        """Test saludos usan LLM directo."""
        result = classifier.classify("hola buenos días")
        
        assert result.strategy == QueryStrategy.LLM_DIRECT


class TestClassifierPatterns:
    """Tests para los patrones del clasificador."""
    
    def test_math_patterns_compiled(self):
        """Test que los patrones de matemáticas están compilados."""
        classifier = QueryClassifier()
        assert len(classifier._math_re) > 0
    
    def test_code_patterns_compiled(self):
        """Test que los patrones de código están compilados."""
        classifier = QueryClassifier()
        assert len(classifier._code_re) > 0
    
    def test_all_patterns_are_valid_regex(self):
        """Test que todos los patrones son regex válidos."""
        import re
        
        all_patterns = (
            QueryClassifier.MATH_PATTERNS +
            QueryClassifier.CODE_PATTERNS +
            QueryClassifier.REASONING_PATTERNS +
            QueryClassifier.TOOLS_PATTERNS +
            QueryClassifier.GREETING_PATTERNS +
            QueryClassifier.FACTUAL_PATTERNS
        )
        
        for pattern in all_patterns:
            try:
                re.compile(pattern, re.IGNORECASE)
            except re.error as e:
                pytest.fail(f"Invalid pattern '{pattern}': {e}")
