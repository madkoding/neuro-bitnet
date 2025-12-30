"""
In-Memory Storage Backend
=========================

Almacenamiento en memoria RAM. Rápido pero volátil.
Ideal para tests y desarrollo.
"""

import hashlib
from datetime import datetime
from typing import List, Dict, Any, Optional, Tuple

import numpy as np

from src.rag.core import Document, SearchResult
from src.rag.storage.base import ABCStorage


class InMemoryStorage(ABCStorage):
    """
    Backend de almacenamiento en memoria.
    
    Características:
    - Muy rápido (todo en RAM)
    - Volátil (se pierde al reiniciar)
    - Sin dependencias externas
    - Ideal para tests y prototipos
    
    Example:
        storage = InMemoryStorage()
        storage.initialize("user1")
        doc_id = storage.add_document(Document(...))
        results = storage.search(embedding, "user1", top_k=3)
    """
    
    def __init__(self):
        # {user_id: {'documents': [...], 'embeddings': np.array}}
        self._data: Dict[str, Dict[str, Any]] = {}
    
    def _get_user_data(self, user_id: str) -> Tuple[List[Document], Optional[np.ndarray]]:
        """Obtiene datos de un usuario, inicializando si no existe."""
        if user_id not in self._data:
            self._data[user_id] = {'documents': [], 'embeddings': None}
        data = self._data[user_id]
        return data['documents'], data['embeddings']
    
    def _set_user_data(
        self, 
        user_id: str, 
        documents: List[Document], 
        embeddings: Optional[np.ndarray]
    ) -> None:
        """Guarda datos de un usuario."""
        self._data[user_id] = {
            'documents': documents,
            'embeddings': embeddings
        }
    
    def initialize(self, user_id: str) -> None:
        """Inicializa espacio para un usuario."""
        if user_id not in self._data:
            self._data[user_id] = {'documents': [], 'embeddings': None}
    
    def add_document(self, doc: Document) -> str:
        """Agrega documento con su embedding."""
        documents, embeddings = self._get_user_data(doc.user_id)
        
        # Generar ID si no tiene
        if not doc.id:
            doc.id = hashlib.md5(
                f"{doc.content}{datetime.now().isoformat()}".encode()
            ).hexdigest()[:12]
        
        documents.append(doc)
        
        # Actualizar matriz de embeddings
        if doc.embedding is not None:
            new_emb = np.array([doc.embedding])
            if embeddings is None:
                embeddings = new_emb
            else:
                embeddings = np.vstack([embeddings, new_emb])
        
        self._set_user_data(doc.user_id, documents, embeddings)
        return doc.id
    
    def search(
        self, 
        embedding: List[float], 
        user_id: str, 
        top_k: int = 5,
        min_score: float = 0.3
    ) -> List[SearchResult]:
        """Búsqueda por similitud coseno."""
        documents, embeddings = self._get_user_data(user_id)
        
        if not documents or embeddings is None:
            return []
        
        # Calcular similitud coseno
        query = np.array(embedding)
        norms = np.linalg.norm(embeddings, axis=1) * np.linalg.norm(query)
        # Evitar división por cero
        norms = np.where(norms == 0, 1e-10, norms)
        similarities = np.dot(embeddings, query) / norms
        
        # Obtener top-k
        top_indices = np.argsort(similarities)[-top_k:][::-1]
        
        results = []
        for idx in top_indices:
            score = float(similarities[idx])
            if score >= min_score:
                results.append(SearchResult(
                    document=documents[idx],
                    score=score
                ))
        
        return results
    
    def list_documents(self, user_id: str, limit: int = 50) -> List[Document]:
        """Lista documentos de un usuario."""
        documents, _ = self._get_user_data(user_id)
        return documents[:limit]
    
    def get_document(self, doc_id: str, user_id: str) -> Optional[Document]:
        """Obtiene documento por ID."""
        documents, _ = self._get_user_data(user_id)
        for doc in documents:
            if doc.id == doc_id:
                return doc
        return None
    
    def delete_document(self, doc_id: str, user_id: str) -> bool:
        """Elimina un documento y su embedding."""
        documents, embeddings = self._get_user_data(user_id)
        
        for i, doc in enumerate(documents):
            if doc.id == doc_id:
                documents.pop(i)
                if embeddings is not None and len(embeddings) > i:
                    embeddings = np.delete(embeddings, i, axis=0)
                    if len(embeddings) == 0:
                        embeddings = None
                self._set_user_data(user_id, documents, embeddings)
                return True
        
        return False
    
    def clear(self, user_id: str) -> int:
        """Elimina todos los documentos de un usuario."""
        documents, _ = self._get_user_data(user_id)
        count = len(documents)
        self._set_user_data(user_id, [], None)
        return count
    
    def get_stats(self, user_id: str) -> Dict[str, Any]:
        """Estadísticas del almacenamiento."""
        documents, embeddings = self._get_user_data(user_id)
        
        by_source: Dict[str, int] = {}
        for doc in documents:
            by_source[doc.source] = by_source.get(doc.source, 0) + 1
        
        return {
            "total": len(documents),
            "by_source": by_source,
            "has_embeddings": embeddings is not None,
            "embedding_count": len(embeddings) if embeddings is not None else 0
        }
