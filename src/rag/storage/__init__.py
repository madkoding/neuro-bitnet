"""
Storage Module
==============

Backends de almacenamiento para documentos RAG.
"""

from src.rag.storage.base import ABCStorage
from src.rag.storage.memory import InMemoryStorage
from src.rag.storage.files import FileStorage

__all__ = [
    'ABCStorage',
    'InMemoryStorage', 
    'FileStorage',
]
