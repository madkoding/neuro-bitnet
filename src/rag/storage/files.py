"""
File Storage Backend
====================

Almacenamiento persistente en archivos locales.
"""

import json
import hashlib
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Optional, Tuple

import numpy as np

from src.rag.core import Document, SearchResult, DEFAULT_DATA_DIR
from src.rag.storage.base import ABCStorage


class FileStorage(ABCStorage):
    """
    Backend de almacenamiento en archivos locales.
    
    Estructura:
        ~/.neuro-bitnet/rag/
        ├── user1/
        │   ├── documents.json
        │   └── embeddings.npy
        └── user2/
            ├── documents.json
            └── embeddings.npy
    
    Características:
    - Persistente entre reinicios
    - Sin dependencias externas (solo numpy)
    - Cache en memoria para rendimiento
    - Un directorio por usuario
    
    Example:
        storage = FileStorage()
        storage.initialize("user1")
        doc_id = storage.add_document(Document(...))
    """
    
    def __init__(self, data_dir: Optional[Path] = None):
        """
        Args:
            data_dir: Directorio base para almacenamiento.
                     Default: ~/.neuro-bitnet/rag/
        """
        self.data_dir = data_dir or DEFAULT_DATA_DIR
        # Cache en memoria: {user_id: (documents, embeddings)}
        self._cache: Dict[str, Tuple[List[Document], Optional[np.ndarray]]] = {}
    
    def _get_user_path(self, user_id: str) -> Path:
        """Retorna el path del directorio de un usuario."""
        return self.data_dir / user_id
    
    def _load_user_data(self, user_id: str) -> Tuple[List[Document], Optional[np.ndarray]]:
        """
        Carga datos de un usuario desde disco (o cache).
        
        Returns:
            Tupla (lista de documentos, matriz de embeddings)
        """
        if user_id in self._cache:
            return self._cache[user_id]
        
        user_path = self._get_user_path(user_id)
        docs_file = user_path / "documents.json"
        emb_file = user_path / "embeddings.npy"
        
        documents: List[Document] = []
        embeddings: Optional[np.ndarray] = None
        
        if docs_file.exists():
            with open(docs_file, 'r', encoding='utf-8') as f:
                docs_data = json.load(f)
                documents = [Document.from_dict(d) for d in docs_data]
        
        if emb_file.exists():
            embeddings = np.load(emb_file)
        
        self._cache[user_id] = (documents, embeddings)
        return documents, embeddings
    
    def _save_user_data(
        self, 
        user_id: str, 
        documents: List[Document], 
        embeddings: Optional[np.ndarray]
    ) -> None:
        """
        Guarda datos de un usuario a disco y actualiza cache.
        """
        user_path = self._get_user_path(user_id)
        user_path.mkdir(parents=True, exist_ok=True)
        
        # Guardar documentos (sin embeddings para reducir tamaño)
        docs_data = [d.to_dict(include_embedding=False) for d in documents]
        with open(user_path / "documents.json", 'w', encoding='utf-8') as f:
            json.dump(docs_data, f, ensure_ascii=False, indent=2)
        
        # Guardar embeddings
        if embeddings is not None and len(embeddings) > 0:
            np.save(user_path / "embeddings.npy", np.array(embeddings))
        else:
            # Eliminar archivo si no hay embeddings
            emb_file = user_path / "embeddings.npy"
            if emb_file.exists():
                emb_file.unlink()
        
        # Actualizar cache
        self._cache[user_id] = (documents, embeddings)
    
    def initialize(self, user_id: str) -> None:
        """Crea el directorio del usuario si no existe."""
        user_path = self._get_user_path(user_id)
        user_path.mkdir(parents=True, exist_ok=True)
    
    def add_document(self, doc: Document) -> str:
        """Agrega documento con su embedding."""
        documents, embeddings = self._load_user_data(doc.user_id)
        
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
        
        self._save_user_data(doc.user_id, documents, embeddings)
        return doc.id
    
    def search(
        self, 
        embedding: List[float], 
        user_id: str, 
        top_k: int = 5,
        min_score: float = 0.3
    ) -> List[SearchResult]:
        """Búsqueda por similitud coseno."""
        documents, embeddings = self._load_user_data(user_id)
        
        if not documents or embeddings is None:
            return []
        
        # Calcular similitud coseno
        query = np.array(embedding)
        norms = np.linalg.norm(embeddings, axis=1) * np.linalg.norm(query)
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
        documents, _ = self._load_user_data(user_id)
        return documents[:limit]
    
    def get_document(self, doc_id: str, user_id: str) -> Optional[Document]:
        """Obtiene documento por ID."""
        documents, _ = self._load_user_data(user_id)
        for doc in documents:
            if doc.id == doc_id:
                return doc
        return None
    
    def delete_document(self, doc_id: str, user_id: str) -> bool:
        """Elimina un documento y su embedding."""
        documents, embeddings = self._load_user_data(user_id)
        
        for i, doc in enumerate(documents):
            if doc.id == doc_id:
                documents.pop(i)
                if embeddings is not None and len(embeddings) > i:
                    embeddings = np.delete(embeddings, i, axis=0)
                    if len(embeddings) == 0:
                        embeddings = None
                self._save_user_data(user_id, documents, embeddings)
                return True
        
        return False
    
    def clear(self, user_id: str) -> int:
        """Elimina todos los documentos de un usuario."""
        documents, _ = self._load_user_data(user_id)
        count = len(documents)
        self._save_user_data(user_id, [], None)
        return count
    
    def get_stats(self, user_id: str) -> Dict[str, Any]:
        """Estadísticas del almacenamiento."""
        documents, embeddings = self._load_user_data(user_id)
        
        by_source: Dict[str, int] = {}
        for doc in documents:
            by_source[doc.source] = by_source.get(doc.source, 0) + 1
        
        user_path = self._get_user_path(user_id)
        
        return {
            "total": len(documents),
            "by_source": by_source,
            "has_embeddings": embeddings is not None,
            "embedding_count": len(embeddings) if embeddings is not None else 0,
            "storage_path": str(user_path)
        }
    
    def invalidate_cache(self, user_id: Optional[str] = None) -> None:
        """
        Invalida el cache para forzar recarga desde disco.
        
        Args:
            user_id: Usuario específico o None para invalidar todo
        """
        if user_id:
            self._cache.pop(user_id, None)
        else:
            self._cache.clear()
