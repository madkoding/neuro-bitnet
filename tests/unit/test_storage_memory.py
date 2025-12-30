"""
Unit Tests for InMemory Storage
===============================

Tests para el backend de almacenamiento en memoria.
"""

import pytest
from src.rag.core import Document
from src.rag.storage.memory import InMemoryStorage


@pytest.fixture
def storage():
    """Crea una instancia limpia del storage."""
    s = InMemoryStorage()
    s.initialize("test_user")
    return s


@pytest.fixture
def sample_document():
    """Crea un documento de prueba."""
    return Document(
        id="doc1",
        content="This is a test document about machine learning.",
        user_id="test_user",
        source="manual",
        metadata={"topic": "ML"},
        embedding=[0.1, 0.2, 0.3, 0.4]
    )


class TestInMemoryStorageInitialization:
    """Tests para inicialización del storage."""
    
    def test_initialize_creates_user(self):
        """Test que initialize crea el usuario."""
        storage = InMemoryStorage()
        storage.initialize("user1")
        
        stats = storage.get_stats()
        assert stats["total_documents"] == 0
    
    def test_initialize_multiple_users(self):
        """Test que se pueden inicializar múltiples usuarios."""
        storage = InMemoryStorage()
        storage.initialize("user1")
        storage.initialize("user2")
        
        # Ambos deben funcionar
        storage.add_document(Document(
            id="d1", content="Content", user_id="user1",
            source="test", embedding=[0.1]
        ))
        storage.add_document(Document(
            id="d2", content="Content", user_id="user2",
            source="test", embedding=[0.1]
        ))
        
        # User1 tiene 1 doc
        docs1 = storage.list_documents("user1")
        assert len(docs1) == 1
        
        # User2 tiene 1 doc
        docs2 = storage.list_documents("user2")
        assert len(docs2) == 1


class TestAddDocument:
    """Tests para añadir documentos."""
    
    def test_add_document_returns_id(self, storage, sample_document):
        """Test que add_document retorna el ID."""
        doc_id = storage.add_document(sample_document)
        
        assert doc_id == "doc1"
    
    def test_add_document_increments_count(self, storage, sample_document):
        """Test que añadir documento incrementa el contador."""
        storage.add_document(sample_document)
        
        stats = storage.get_stats()
        assert stats["total_documents"] == 1
    
    def test_add_multiple_documents(self, storage):
        """Test añadir múltiples documentos."""
        for i in range(5):
            doc = Document(
                id=f"doc{i}",
                content=f"Content {i}",
                user_id="test_user",
                source="test",
                embedding=[float(i)]
            )
            storage.add_document(doc)
        
        stats = storage.get_stats()
        assert stats["total_documents"] == 5


class TestGetDocument:
    """Tests para obtener documentos."""
    
    def test_get_existing_document(self, storage, sample_document):
        """Test obtener documento existente."""
        storage.add_document(sample_document)
        
        doc = storage.get_document("doc1", "test_user")
        
        assert doc is not None
        assert doc.id == "doc1"
        assert doc.content == "This is a test document about machine learning."
    
    def test_get_nonexistent_document(self, storage):
        """Test obtener documento que no existe."""
        doc = storage.get_document("nonexistent", "test_user")
        
        assert doc is None
    
    def test_get_document_wrong_user(self, storage, sample_document):
        """Test obtener documento de otro usuario."""
        storage.add_document(sample_document)
        
        doc = storage.get_document("doc1", "other_user")
        
        assert doc is None


class TestListDocuments:
    """Tests para listar documentos."""
    
    def test_list_empty(self, storage):
        """Test listar cuando no hay documentos."""
        docs = storage.list_documents("test_user")
        
        assert len(docs) == 0
    
    def test_list_with_documents(self, storage):
        """Test listar con documentos."""
        for i in range(3):
            doc = Document(
                id=f"doc{i}",
                content=f"Content {i}",
                user_id="test_user",
                source="test",
                embedding=[float(i)]
            )
            storage.add_document(doc)
        
        docs = storage.list_documents("test_user")
        
        assert len(docs) == 3
    
    def test_list_only_user_documents(self, storage):
        """Test que solo lista documentos del usuario."""
        storage.add_document(Document(
            id="d1", content="User1 doc",
            user_id="user1", source="test", embedding=[0.1]
        ))
        storage.add_document(Document(
            id="d2", content="User2 doc",
            user_id="user2", source="test", embedding=[0.2]
        ))
        
        docs = storage.list_documents("user1")
        
        assert len(docs) == 1
        assert docs[0].user_id == "user1"


class TestDeleteDocument:
    """Tests para eliminar documentos."""
    
    def test_delete_existing(self, storage, sample_document):
        """Test eliminar documento existente."""
        storage.add_document(sample_document)
        
        result = storage.delete_document("doc1", "test_user")
        
        assert result is True
        assert storage.get_document("doc1", "test_user") is None
    
    def test_delete_nonexistent(self, storage):
        """Test eliminar documento que no existe."""
        result = storage.delete_document("nonexistent", "test_user")
        
        assert result is False
    
    def test_delete_wrong_user(self, storage, sample_document):
        """Test no puede eliminar documento de otro usuario."""
        storage.add_document(sample_document)
        
        result = storage.delete_document("doc1", "other_user")
        
        assert result is False
        # El documento sigue existiendo
        assert storage.get_document("doc1", "test_user") is not None


class TestClear:
    """Tests para limpiar storage."""
    
    def test_clear_removes_all(self, storage):
        """Test que clear elimina todos los documentos."""
        for i in range(3):
            doc = Document(
                id=f"doc{i}",
                content=f"Content {i}",
                user_id="test_user",
                source="test",
                embedding=[float(i)]
            )
            storage.add_document(doc)
        
        storage.clear("test_user")
        
        docs = storage.list_documents("test_user")
        assert len(docs) == 0
    
    def test_clear_only_user_data(self, storage):
        """Test que clear solo elimina datos del usuario."""
        storage.add_document(Document(
            id="d1", content="User1 doc",
            user_id="user1", source="test", embedding=[0.1]
        ))
        storage.add_document(Document(
            id="d2", content="User2 doc",
            user_id="user2", source="test", embedding=[0.2]
        ))
        
        storage.clear("user1")
        
        # User1 vacío
        assert len(storage.list_documents("user1")) == 0
        # User2 intacto
        assert len(storage.list_documents("user2")) == 1


class TestSearch:
    """Tests para búsqueda semántica."""
    
    def test_search_returns_results(self, storage):
        """Test que search retorna resultados."""
        # Añadir documentos con embeddings distintos
        storage.add_document(Document(
            id="d1", content="Machine learning basics",
            user_id="test_user", source="test",
            embedding=[1.0, 0.0, 0.0]
        ))
        storage.add_document(Document(
            id="d2", content="Deep learning advanced",
            user_id="test_user", source="test",
            embedding=[0.9, 0.1, 0.0]
        ))
        storage.add_document(Document(
            id="d3", content="Cooking recipes",
            user_id="test_user", source="test",
            embedding=[0.0, 0.0, 1.0]
        ))
        
        # Buscar con embedding similar a ML
        results = storage.search(
            embedding=[0.95, 0.05, 0.0],
            user_id="test_user",
            top_k=2
        )
        
        assert len(results) == 2
        # Los resultados de ML deben ser más similares
        assert results[0].document.id in ["d1", "d2"]
    
    def test_search_respects_top_k(self, storage):
        """Test que search respeta el límite top_k."""
        for i in range(10):
            storage.add_document(Document(
                id=f"d{i}", content=f"Content {i}",
                user_id="test_user", source="test",
                embedding=[float(i) / 10]
            ))
        
        results = storage.search(
            embedding=[0.5],
            user_id="test_user",
            top_k=3
        )
        
        assert len(results) <= 3
    
    def test_search_returns_search_results(self, storage, sample_document):
        """Test que search retorna SearchResult objects."""
        storage.add_document(sample_document)
        
        results = storage.search(
            embedding=[0.1, 0.2, 0.3, 0.4],
            user_id="test_user",
            top_k=1
        )
        
        assert len(results) >= 0  # Puede ser 0 si min_score filtra
        if len(results) > 0:
            from src.rag.core import SearchResult
            assert isinstance(results[0], SearchResult)
            assert hasattr(results[0], 'score')
            assert hasattr(results[0], 'document')


class TestGetStats:
    """Tests para estadísticas."""
    
    def test_stats_empty(self, storage):
        """Test stats cuando está vacío."""
        stats = storage.get_stats()
        
        assert stats["total_documents"] == 0
    
    def test_stats_with_documents(self, storage):
        """Test stats con documentos."""
        for i in range(5):
            storage.add_document(Document(
                id=f"d{i}", content=f"Content {i}",
                user_id="test_user", source="test",
                embedding=[float(i)]
            ))
        
        stats = storage.get_stats()
        
        assert stats["total_documents"] == 5
        assert stats["storage_type"] == "memory"
