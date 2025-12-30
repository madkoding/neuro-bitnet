"""
Unit Tests for Storage Base Module
===================================

Tests para ABCStorage interface.
"""

import pytest
from abc import ABC
from src.rag.storage.base import ABCStorage


class TestABCStorage:
    """Tests para la clase ABCStorage."""
    
    def test_is_abstract_class(self):
        """Test que ABCStorage es una clase abstracta."""
        assert issubclass(ABCStorage, ABC)
    
    def test_cannot_instantiate_directly(self):
        """Test que no se puede instanciar directamente."""
        with pytest.raises(TypeError):
            ABCStorage()
    
    def test_abstract_methods_exist(self):
        """Test que los métodos abstractos están definidos."""
        abstract_methods = ABCStorage.__abstractmethods__
        
        expected_methods = {
            'add_document',
            'search',
            'list_documents',
            'get_document',
            'delete_document',
            'clear',
            'get_stats'
        }
        
        assert expected_methods == abstract_methods


class TestConcreteStorageImplementation:
    """Tests para verificar que las implementaciones concretas funcionan."""
    
    def test_inmemory_storage_is_valid_implementation(self):
        """Test que InMemoryStorage implementa ABCStorage."""
        from src.rag.storage.memory import InMemoryStorage
        
        storage = InMemoryStorage()
        
        assert isinstance(storage, ABCStorage)
    
    def test_file_storage_is_valid_implementation(self, tmp_path):
        """Test que FileStorage implementa ABCStorage."""
        from src.rag.storage.files import FileStorage
        
        storage = FileStorage(tmp_path)
        
        assert isinstance(storage, ABCStorage)
