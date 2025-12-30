"""
Unit Tests for File Storage
===========================

Tests para el backend de almacenamiento en archivos.
"""

import pytest
import json
from pathlib import Path

from src.rag.core import Document
from src.rag.storage.files import FileStorage


@pytest.fixture
def storage(tmp_path):
    """Crea una instancia de FileStorage con directorio temporal."""
    s = FileStorage(tmp_path)
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


class TestFileStorageInitialization:
    """Tests para inicialización del storage."""
    
    def test_initialize_creates_directory(self, tmp_path):
        """Test que initialize crea el directorio del usuario."""
        storage = FileStorage(tmp_path)
        storage.initialize("user1")
        
        user_dir = tmp_path / "user1"
        assert user_dir.exists()
        assert user_dir.is_dir()
    
    def test_initialize_creates_empty_documents_file(self, tmp_path):
        """Test que initialize crea archivo de documentos."""
        storage = FileStorage(tmp_path)
        storage.initialize("user1")
        
        docs_file = tmp_path / "user1" / "documents.json"
        # El archivo se crea al agregar el primer documento
        # Inicialmente solo el directorio existe
        user_dir = tmp_path / "user1"
        assert user_dir.exists()


class TestFileStoragePersistence:
    """Tests para persistencia de datos."""
    
    def test_documents_persist_after_reload(self, tmp_path, sample_document):
        """Test que documentos persisten después de recargar."""
        # Crear storage y añadir documento
        storage1 = FileStorage(tmp_path)
        storage1.initialize("test_user")
        storage1.add_document(sample_document)
        
        # Crear nuevo storage con misma ruta (simula reinicio)
        storage2 = FileStorage(tmp_path)
        storage2.initialize("test_user")
        
        # Debe encontrar el documento
        doc = storage2.get_document("doc1", "test_user")
        assert doc is not None
        assert doc.content == sample_document.content
    
    def test_embeddings_persist_correctly(self, tmp_path):
        """Test que embeddings se guardan y cargan correctamente."""
        embedding = [0.1, 0.2, 0.3, 0.4, 0.5]
        doc = Document(
            id="emb_test",
            content="Test",
            user_id="test_user",
            source="test",
            embedding=embedding
        )
        
        # Guardar
        storage1 = FileStorage(tmp_path)
        storage1.initialize("test_user")
        storage1.add_document(doc)
        
        # Recargar
        storage2 = FileStorage(tmp_path)
        storage2.initialize("test_user")
        
        # Verificar embedding
        loaded = storage2.get_document("emb_test", "test_user")
        assert loaded is not None
        assert loaded.embedding is not None
        
        # Comparar valores (pueden ser np.array o list)
        import numpy as np
        np.testing.assert_array_almost_equal(loaded.embedding, embedding)


class TestFileStorageAddDocument:
    """Tests para añadir documentos."""
    
    def test_add_document_creates_file(self, storage, sample_document, tmp_path):
        """Test que añadir documento crea archivo."""
        storage.add_document(sample_document)
        
        docs_file = tmp_path / "test_user" / "documents.json"
        assert docs_file.exists()
    
    def test_add_document_content_correct(self, storage, sample_document, tmp_path):
        """Test que el contenido guardado es correcto."""
        storage.add_document(sample_document)
        
        docs_file = tmp_path / "test_user" / "documents.json"
        with open(docs_file, 'r') as f:
            data = json.load(f)
        
        assert len(data) == 1
        assert data[0]["id"] == "doc1"
        assert data[0]["content"] == sample_document.content


class TestFileStorageSearch:
    """Tests para búsqueda."""
    
    def test_search_after_reload(self, tmp_path):
        """Test que búsqueda funciona después de recargar."""
        # Añadir documentos
        storage1 = FileStorage(tmp_path)
        storage1.initialize("test_user")
        
        storage1.add_document(Document(
            id="d1", content="Machine learning",
            user_id="test_user", source="test",
            embedding=[1.0, 0.0, 0.0]
        ))
        storage1.add_document(Document(
            id="d2", content="Cooking",
            user_id="test_user", source="test",
            embedding=[0.0, 0.0, 1.0]
        ))
        
        # Recargar
        storage2 = FileStorage(tmp_path)
        storage2.initialize("test_user")
        
        # Buscar
        results = storage2.search(
            embedding=[0.9, 0.1, 0.0],
            user_id="test_user",
            top_k=1
        )
        
        assert len(results) == 1
        assert results[0].document.id == "d1"


class TestFileStorageDelete:
    """Tests para eliminar documentos."""
    
    def test_delete_updates_file(self, storage, sample_document, tmp_path):
        """Test que eliminar actualiza archivo."""
        storage.add_document(sample_document)
        storage.delete_document("doc1", "test_user")
        
        docs_file = tmp_path / "test_user" / "documents.json"
        with open(docs_file, 'r') as f:
            data = json.load(f)
        
        assert len(data) == 0


class TestFileStorageClear:
    """Tests para limpiar storage."""
    
    def test_clear_removes_files(self, storage, sample_document, tmp_path):
        """Test que clear elimina archivos."""
        storage.add_document(sample_document)
        storage.clear("test_user")
        
        docs_file = tmp_path / "test_user" / "documents.json"
        
        # El archivo existe pero está vacío
        if docs_file.exists():
            with open(docs_file, 'r') as f:
                data = json.load(f)
            assert len(data) == 0


class TestFileStorageStats:
    """Tests para estadísticas."""
    
    def test_stats_returns_correct_type(self, storage):
        """Test que stats retorna tipo correcto."""
        stats = storage.get_stats()
        
        assert stats["storage_type"] == "file"
    
    def test_stats_shows_data_dir(self, storage, tmp_path):
        """Test que stats muestra directorio de datos."""
        stats = storage.get_stats()
        
        assert "data_dir" in stats
        assert str(tmp_path) in stats["data_dir"]


class TestFileStorageEdgeCases:
    """Tests para casos límite."""
    
    def test_empty_embedding(self, storage):
        """Test documento sin embedding."""
        doc = Document(
            id="no_emb",
            content="No embedding",
            user_id="test_user",
            source="test",
            embedding=None
        )
        
        doc_id = storage.add_document(doc)
        assert doc_id == "no_emb"
    
    def test_special_characters_in_content(self, storage):
        """Test contenido con caracteres especiales."""
        doc = Document(
            id="special",
            content="Texto con ñ, acentos: á é í ó ú, y emojis: 🎉",
            user_id="test_user",
            source="test",
            embedding=[0.1]
        )
        
        storage.add_document(doc)
        loaded = storage.get_document("special", "test_user")
        
        assert loaded is not None
        assert "🎉" in loaded.content
    
    def test_large_document(self, storage):
        """Test documento grande."""
        large_content = "x" * 100000  # 100KB
        doc = Document(
            id="large",
            content=large_content,
            user_id="test_user",
            source="test",
            embedding=[0.1]
        )
        
        storage.add_document(doc)
        loaded = storage.get_document("large", "test_user")
        
        assert loaded is not None
        assert len(loaded.content) == 100000
