"""
Abstract Storage Backend
========================

Define la interfaz común para todos los backends de almacenamiento.
"""

from abc import ABC, abstractmethod
from typing import List, Dict, Any, Optional

from src.rag.core import Document, SearchResult


class ABCStorage(ABC):
    """
    Interfaz abstracta para backends de almacenamiento RAG.
    
    Todos los backends deben implementar estos métodos para garantizar
    compatibilidad con el sistema RAG.
    
    Example:
        class MyStorage(ABCStorage):
            def add_document(self, doc: Document) -> str:
                # Implementación específica
                pass
    """
    
    @abstractmethod
    def initialize(self, user_id: str) -> None:
        """
        Inicializa el almacenamiento para un usuario.
        
        Args:
            user_id: Identificador del usuario
        """
        pass
    
    @abstractmethod
    def add_document(self, doc: Document) -> str:
        """
        Agrega un documento al almacenamiento.
        
        Args:
            doc: Documento a almacenar (debe incluir embedding)
            
        Returns:
            ID del documento creado
        """
        pass
    
    @abstractmethod
    def search(
        self, 
        embedding: List[float], 
        user_id: str, 
        top_k: int = 5,
        min_score: float = 0.3
    ) -> List[SearchResult]:
        """
        Busca documentos similares por embedding.
        
        Args:
            embedding: Vector de embedding de la consulta
            user_id: ID del usuario para filtrar documentos
            top_k: Número máximo de resultados
            min_score: Puntuación mínima de similitud (0.0 a 1.0)
            
        Returns:
            Lista de SearchResult ordenados por relevancia
        """
        pass
    
    @abstractmethod
    def list_documents(
        self, 
        user_id: str, 
        limit: int = 50
    ) -> List[Document]:
        """
        Lista documentos de un usuario.
        
        Args:
            user_id: ID del usuario
            limit: Número máximo de documentos a retornar
            
        Returns:
            Lista de documentos
        """
        pass
    
    @abstractmethod
    def get_document(self, doc_id: str, user_id: str) -> Optional[Document]:
        """
        Obtiene un documento por ID.
        
        Args:
            doc_id: ID del documento
            user_id: ID del usuario propietario
            
        Returns:
            Document o None si no existe
        """
        pass
    
    @abstractmethod
    def delete_document(self, doc_id: str, user_id: str) -> bool:
        """
        Elimina un documento.
        
        Args:
            doc_id: ID del documento a eliminar
            user_id: ID del usuario propietario
            
        Returns:
            True si se eliminó, False si no existía
        """
        pass
    
    @abstractmethod
    def clear(self, user_id: str) -> int:
        """
        Elimina todos los documentos de un usuario.
        
        Args:
            user_id: ID del usuario
            
        Returns:
            Número de documentos eliminados
        """
        pass
    
    @abstractmethod
    def get_stats(self, user_id: str) -> Dict[str, Any]:
        """
        Obtiene estadísticas del almacenamiento.
        
        Args:
            user_id: ID del usuario
            
        Returns:
            Diccionario con estadísticas (total, by_source, etc.)
        """
        pass
