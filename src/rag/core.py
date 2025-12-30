"""
Core Data Models and Constants
==============================

Modelos de datos fundamentales y constantes compartidas por todo el sistema RAG.
"""

import os
from pathlib import Path
from datetime import datetime
from dataclasses import dataclass, asdict, field
from typing import Dict, Any, Optional, List


# ============================================================================
# Constants
# ============================================================================

DEFAULT_LLM_URL = os.getenv("RAG_LLM_URL", "http://localhost:11435")
DEFAULT_RAG_PORT = int(os.getenv("RAG_SERVER_PORT", "11436"))
DEFAULT_DATA_DIR = Path.home() / ".neuro-bitnet" / "rag"

# SurrealDB settings
DEFAULT_SURREALDB_URL = os.getenv("RAG_SURREALDB_URL", "ws://localhost:8000/rpc")
DEFAULT_SURREALDB_USER = os.getenv("RAG_SURREALDB_USER", "root")
DEFAULT_SURREALDB_PASS = os.getenv("RAG_SURREALDB_PASS", "root")
DEFAULT_DATABASE = "neurobitnet"
DEFAULT_NAMESPACE = "rag"

# Embedding models with short aliases
EMBEDDING_MODELS = {
    "minilm": {
        "name": "sentence-transformers/all-MiniLM-L6-v2",
        "dim": 384,
        "description": "Rápido y ligero (80MB)",
    },
    "mpnet": {
        "name": "sentence-transformers/all-mpnet-base-v2",
        "dim": 768,
        "description": "Mejor calidad (420MB)",
    },
    "e5": {
        "name": "intfloat/e5-large-v2",
        "dim": 1024,
        "description": "Excelente para búsqueda (1.2GB)",
    },
    "bge": {
        "name": "BAAI/bge-large-en-v1.5",
        "dim": 1024,
        "description": "Multiidioma (1.3GB)",
    },
}


# ============================================================================
# Data Classes
# ============================================================================

@dataclass
class Document:
    """
    Documento almacenado en el RAG.
    
    Attributes:
        id: Identificador único del documento
        content: Contenido textual del documento
        user_id: ID del usuario propietario
        source: Origen del documento (manual, file, web, conversation)
        metadata: Metadatos adicionales
        created_at: Fecha de creación ISO 8601
        embedding: Vector de embedding (opcional, no se serializa)
    """
    id: str
    content: str
    user_id: str
    source: str
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())
    embedding: Optional[List[float]] = None

    def to_dict(self, include_embedding: bool = False) -> Dict[str, Any]:
        """Convierte a diccionario, opcionalmente sin embedding."""
        d = asdict(self)
        if not include_embedding:
            d['embedding'] = None
        return d
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Document':
        """Crea un Document desde un diccionario."""
        return cls(**{k: v for k, v in data.items() if k in cls.__dataclass_fields__})


@dataclass
class SearchResult:
    """
    Resultado de búsqueda con puntuación de relevancia.
    
    Attributes:
        document: Documento encontrado
        score: Puntuación de similitud (0.0 a 1.0)
    """
    document: Document
    score: float
    
    def to_dict(self) -> Dict[str, Any]:
        """Convierte a diccionario serializable."""
        return {
            "id": self.document.id,
            "content": self.document.content,
            "source": self.document.source,
            "score": self.score,
            "metadata": self.document.metadata,
        }


@dataclass
class QueryResult:
    """
    Resultado completo de una consulta RAG.
    
    Attributes:
        answer: Respuesta generada por el LLM
        sources: Lista de fuentes usadas
        classification: Resultado de clasificación de la consulta
        timing_ms: Tiempo de procesamiento en milisegundos
    """
    answer: str
    sources: List[Dict[str, Any]]
    classification: Dict[str, Any]
    timing_ms: int
    
    def to_dict(self) -> Dict[str, Any]:
        """Convierte a diccionario serializable."""
        return {
            "answer": self.answer,
            "sources": self.sources,
            "classification": self.classification,
            "timing": {"total_ms": self.timing_ms}
        }
