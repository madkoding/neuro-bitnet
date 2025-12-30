"""
Integration Tests for RAG Pipeline
===================================

Tests de integración para el pipeline completo de RAG.
"""

import pytest
from pathlib import Path

from src.rag.core import Document
from src.rag.classifier import QueryClassifier, QueryCategory, QueryStrategy
from src.rag.storage.memory import InMemoryStorage
from src.rag.indexer.python import PythonAnalyzer
from src.rag.indexer.generic import GenericAnalyzer


class TestClassifierStorageIntegration:
    """Tests de integración entre Classifier y Storage."""
    
    @pytest.fixture
    def storage(self):
        """Storage con documentos de prueba."""
        s = InMemoryStorage()
        s.initialize("test_user")
        
        # Añadir documentos de diferentes tipos
        docs = [
            Document(
                id="doc_ml",
                content="Machine learning is a subset of artificial intelligence.",
                user_id="test_user",
                source="manual",
                metadata={"topic": "ML"},
                embedding=[1.0, 0.0, 0.0]
            ),
            Document(
                id="doc_python",
                content="Python is a programming language used for data science.",
                user_id="test_user",
                source="manual",
                metadata={"topic": "programming"},
                embedding=[0.5, 0.5, 0.0]
            ),
            Document(
                id="doc_history",
                content="Albert Einstein developed the theory of relativity in 1905.",
                user_id="test_user",
                source="manual",
                metadata={"topic": "history"},
                embedding=[0.0, 0.0, 1.0]
            ),
        ]
        
        for doc in docs:
            s.add_document(doc)
        
        return s
    
    @pytest.fixture
    def classifier(self):
        """Clasificador de consultas."""
        return QueryClassifier()
    
    def test_factual_query_uses_rag(self, classifier, storage):
        """Test que consulta factual usa RAG para buscar."""
        query = "quién fue Albert Einstein"
        
        # Clasificar
        result = classifier.classify(query)
        assert result.category == QueryCategory.FACTUAL
        assert result.strategy == QueryStrategy.RAG_THEN_WEB
        
        # Buscar (simulando embedding de la consulta)
        results = storage.search(
            embedding=[0.1, 0.0, 0.9],  # Similar a doc_history
            user_id="test_user",
            top_k=3
        )
        
        # Debe encontrar el documento de Einstein
        assert len(results) > 0
        assert any("Einstein" in r.document.content for r in results)
    
    def test_code_query_uses_rag_local(self, classifier, storage):
        """Test que consulta de código usa RAG local."""
        query = "cómo programar en python"
        
        result = classifier.classify(query)
        assert result.category == QueryCategory.CODE
        assert result.strategy == QueryStrategy.RAG_LOCAL
    
    def test_math_query_skips_rag(self, classifier, storage):
        """Test que consulta matemática no necesita RAG."""
        query = "5 + 3"
        
        result = classifier.classify(query)
        assert result.category == QueryCategory.MATH
        assert result.strategy == QueryStrategy.LLM_DIRECT


class TestIndexerStorageIntegration:
    """Tests de integración entre Indexers y Storage."""
    
    @pytest.fixture
    def storage(self):
        """Storage vacío."""
        s = InMemoryStorage()
        s.initialize("test_user")
        return s
    
    @pytest.fixture
    def python_analyzer(self):
        return PythonAnalyzer()
    
    @pytest.fixture
    def generic_analyzer(self):
        return GenericAnalyzer()
    
    def test_python_code_indexed_and_searchable(self, storage, python_analyzer):
        """Test que código Python se indexa y es buscable."""
        code = '''
def calculate_fibonacci(n: int) -> int:
    """Calculate the nth Fibonacci number."""
    if n <= 1:
        return n
    return calculate_fibonacci(n-1) + calculate_fibonacci(n-2)

class MathUtils:
    """Math utility functions."""
    
    @staticmethod
    def factorial(n: int) -> int:
        """Calculate factorial of n."""
        if n <= 1:
            return 1
        return n * MathUtils.factorial(n-1)
'''
        
        # Analizar código
        elements = python_analyzer.analyze("math_utils.py", code)
        
        # Indexar elements como documentos
        for elem in elements:
            doc = Document(
                id=f"chunk_{elem.name}",
                content=elem.code_snippet,
                user_id="test_user",
                source="math_utils.py",
                metadata={
                    "type": elem.type,
                    "name": elem.name,
                    "line_number": elem.line_number
                },
                embedding=[0.1] * 10  # Embedding dummy
            )
            storage.add_document(doc)
        
        # Verificar que se indexaron
        docs = storage.list_documents("test_user")
        assert len(docs) >= 2  # Al menos fibonacci y MathUtils
        
        # Verificar metadata
        func_docs = [d for d in docs if d.metadata.get("type") == "function"]
        class_docs = [d for d in docs if d.metadata.get("type") == "class"]
        
        assert len(func_docs) >= 1
        assert len(class_docs) >= 1
    
    def test_javascript_code_indexed(self, storage, generic_analyzer):
        """Test que código JS se indexa correctamente."""
        code = '''
class Calculator {
    constructor(initial = 0) {
        this.value = initial;
    }
    
    add(n) {
        this.value += n;
        return this;
    }
    
    multiply(n) {
        this.value *= n;
        return this;
    }
}

function formatNumber(n) {
    return n.toLocaleString();
}
'''
        
        elements = generic_analyzer.analyze("calculator.js", code)
        
        for i, elem in enumerate(elements):
            doc = Document(
                id=f"js_chunk_{i}",
                content=elem.code_snippet,
                user_id="test_user",
                source="calculator.js",
                metadata={"type": elem.type, "name": elem.name},
                embedding=[0.1] * 10
            )
            storage.add_document(doc)
        
        docs = storage.list_documents("test_user")
        assert len(docs) >= 1


class TestMultipleAnalyzersIntegration:
    """Tests de integración con múltiples analizadores."""
    
    @pytest.fixture
    def analyzers(self):
        """Lista de analizadores."""
        return [PythonAnalyzer(), GenericAnalyzer()]
    
    def test_select_correct_analyzer(self, analyzers):
        """Test que se selecciona el analizador correcto."""
        files = {
            "script.py": "python",
            "app.js": "javascript",
            "main.go": "go",
            "lib.rs": "rust",
        }
        
        for filename, expected_lang in files.items():
            # Buscar analizador que puede procesar
            suitable = [a for a in analyzers if a.can_analyze(filename)]
            
            assert len(suitable) >= 1, f"No analyzer for {filename}"
    
    def test_analyze_mixed_project(self, analyzers):
        """Test analizar proyecto con múltiples lenguajes."""
        files = {
            "utils.py": '''
def helper():
    """Python helper."""
    pass
''',
            "app.js": '''
function main() {
    console.log("Hello");
}
''',
        }
        
        all_chunks = []
        
        for filename, code in files.items():
            for analyzer in analyzers:
                if analyzer.can_analyze(filename):
                    chunks = analyzer.analyze(filename, code)
                    all_chunks.extend(chunks)
                    break
        
        assert len(all_chunks) >= 2  # Al menos una de cada archivo


class TestEndToEndPipeline:
    """Tests end-to-end del pipeline completo."""
    
    def test_full_pipeline_code_search(self):
        """Test pipeline completo: indexar → clasificar → buscar."""
        # 1. Setup
        storage = InMemoryStorage()
        storage.initialize("user1")
        classifier = QueryClassifier()
        python_analyzer = PythonAnalyzer()
        
        # 2. Indexar código
        code = '''
def search_documents(query: str, top_k: int = 5):
    """Search for relevant documents using semantic search."""
    embedding = encode_query(query)
    results = vector_store.search(embedding, top_k)
    return results

def encode_query(text: str) -> list:
    """Encode text to embedding vector."""
    return model.encode(text).tolist()
'''
        
        elements = python_analyzer.analyze("search.py", code)
        for i, elem in enumerate(elements):
            doc = Document(
                id=f"search_{i}",
                content=elem.code_snippet,
                user_id="user1",
                source="search.py",
                metadata={"type": elem.type},
                embedding=[float(i) / 10] * 5
            )
            storage.add_document(doc)
        
        # 3. Clasificar query
        query = "cómo funciona la búsqueda de documentos"
        classification = classifier.classify(query)
        
        # Debe ser FACTUAL o CODE
        assert classification.category in [QueryCategory.FACTUAL, QueryCategory.CODE]
        
        # 4. Buscar (si necesita RAG)
        if classification.strategy in [QueryStrategy.RAG_LOCAL, QueryStrategy.RAG_THEN_WEB]:
            results = storage.search(
                embedding=[0.05] * 5,
                user_id="user1",
                top_k=3
            )
            
            # Debe encontrar resultados relevantes
            docs = storage.list_documents("user1")
            assert len(docs) > 0
