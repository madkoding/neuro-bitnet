"""
Unit Tests for Core Module
==========================

Tests para Document, SearchResult y constantes.
"""

import pytest
from datetime import datetime

from src.rag.core import Document, SearchResult, QueryResult, EMBEDDING_MODELS


class TestDocument:
    """Tests para la clase Document."""
    
    def test_create_document(self):
        """Test crear documento con todos los campos."""
        doc = Document(
            id="test123",
            content="Test content",
            user_id="user1",
            source="manual"
        )
        
        assert doc.id == "test123"
        assert doc.content == "Test content"
        assert doc.user_id == "user1"
        assert doc.source == "manual"
        assert doc.metadata == {}
        assert doc.embedding is None
    
    def test_document_with_metadata(self):
        """Test documento con metadatos."""
        doc = Document(
            id="test456",
            content="Content",
            user_id="user1",
            source="file",
            metadata={"filename": "test.py", "lines": 100}
        )
        
        assert doc.metadata["filename"] == "test.py"
        assert doc.metadata["lines"] == 100
    
    def test_document_with_embedding(self):
        """Test documento con embedding."""
        embedding = [0.1, 0.2, 0.3, 0.4]
        doc = Document(
            id="test789",
            content="Content",
            user_id="user1",
            source="manual",
            embedding=embedding
        )
        
        assert doc.embedding == embedding
    
    def test_document_to_dict_without_embedding(self):
        """Test serialización sin embedding."""
        doc = Document(
            id="test",
            content="Content",
            user_id="user1",
            source="manual",
            embedding=[0.1, 0.2]
        )
        
        d = doc.to_dict(include_embedding=False)
        
        assert d["id"] == "test"
        assert d["content"] == "Content"
        assert d["embedding"] is None
    
    def test_document_to_dict_with_embedding(self):
        """Test serialización con embedding."""
        doc = Document(
            id="test",
            content="Content",
            user_id="user1",
            source="manual",
            embedding=[0.1, 0.2]
        )
        
        d = doc.to_dict(include_embedding=True)
        
        assert d["embedding"] == [0.1, 0.2]
    
    def test_document_from_dict(self):
        """Test crear documento desde diccionario."""
        data = {
            "id": "test",
            "content": "Content",
            "user_id": "user1",
            "source": "manual",
            "metadata": {"key": "value"},
            "created_at": "2025-01-01T00:00:00"
        }
        
        doc = Document.from_dict(data)
        
        assert doc.id == "test"
        assert doc.content == "Content"
        assert doc.metadata == {"key": "value"}
    
    def test_document_created_at_auto(self):
        """Test que created_at se genera automáticamente."""
        doc = Document(
            id="test",
            content="Content",
            user_id="user1",
            source="manual"
        )
        
        # Verificar que es una fecha ISO válida
        datetime.fromisoformat(doc.created_at)


class TestSearchResult:
    """Tests para la clase SearchResult."""
    
    def test_create_search_result(self):
        """Test crear resultado de búsqueda."""
        doc = Document(
            id="test",
            content="Content",
            user_id="user1",
            source="manual"
        )
        
        result = SearchResult(document=doc, score=0.85)
        
        assert result.document == doc
        assert result.score == 0.85
    
    def test_search_result_to_dict(self):
        """Test serialización de resultado."""
        doc = Document(
            id="test",
            content="Content",
            user_id="user1",
            source="manual"
        )
        
        result = SearchResult(document=doc, score=0.75)
        d = result.to_dict()
        
        assert d["id"] == "test"
        assert d["content"] == "Content"
        assert d["source"] == "manual"
        assert d["score"] == 0.75


class TestQueryResult:
    """Tests para la clase QueryResult."""
    
    def test_create_query_result(self):
        """Test crear resultado de consulta."""
        result = QueryResult(
            answer="La respuesta es 42",
            sources=[{"source": "web", "title": "Test"}],
            classification={"category": "factual", "strategy": "rag_then_web"},
            timing_ms=150
        )
        
        assert result.answer == "La respuesta es 42"
        assert len(result.sources) == 1
        assert result.timing_ms == 150
    
    def test_query_result_to_dict(self):
        """Test serialización."""
        result = QueryResult(
            answer="Respuesta",
            sources=[],
            classification={"category": "math"},
            timing_ms=100
        )
        
        d = result.to_dict()
        
        assert d["answer"] == "Respuesta"
        assert d["timing"]["total_ms"] == 100


class TestEmbeddingModels:
    """Tests para las constantes de modelos."""
    
    def test_embedding_models_exist(self):
        """Test que los modelos están definidos."""
        assert "minilm" in EMBEDDING_MODELS
        assert "mpnet" in EMBEDDING_MODELS
    
    def test_embedding_model_structure(self):
        """Test estructura de definición de modelo."""
        model = EMBEDDING_MODELS["minilm"]
        
        assert "name" in model
        assert "dim" in model
        assert "description" in model
    
    def test_minilm_dimensions(self):
        """Test dimensiones de minilm."""
        assert EMBEDDING_MODELS["minilm"]["dim"] == 384
    
    def test_mpnet_dimensions(self):
        """Test dimensiones de mpnet."""
        assert EMBEDDING_MODELS["mpnet"]["dim"] == 768
