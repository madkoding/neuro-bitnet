"""
RAG (Retrieval-Augmented Generation) Module
============================================

Este módulo proporciona un sistema RAG inteligente con:
- Clasificación automática de consultas
- Múltiples backends de almacenamiento
- Búsqueda semántica con embeddings
- Indexación de proyectos de código

Ejemplo de uso:
    from src.rag import RAGSystem, QueryClassifier
    
    rag = RAGSystem()
    result = rag.query("¿Cuál es la capital de Francia?")
"""

from src.rag.core import Document, SearchResult, EMBEDDING_MODELS
from src.rag.classifier import QueryClassifier, QueryCategory, QueryStrategy, ClassificationResult
from src.rag.embeddings import EmbeddingsManager
from src.rag.web_search import WebSearcher

__all__ = [
    # Core
    'Document',
    'SearchResult',
    'EMBEDDING_MODELS',
    # Classifier
    'QueryClassifier',
    'QueryCategory', 
    'QueryStrategy',
    'ClassificationResult',
    # Embeddings
    'EmbeddingsManager',
    # Web Search
    'WebSearcher',
]

__version__ = "1.0.0"
