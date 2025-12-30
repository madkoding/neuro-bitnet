"""
Embeddings Manager
==================

Gestiona la generación de embeddings usando sentence-transformers.
Implementado como Singleton para mantener el modelo en memoria.
"""

import time
import logging
import threading
from typing import List, Optional

from src.rag.core import EMBEDDING_MODELS

logger = logging.getLogger(__name__)


class EmbeddingsManager:
    """
    Gestiona embeddings con carga lazy y persistencia en memoria.
    
    Implementado como Singleton thread-safe para evitar cargar
    múltiples instancias del modelo en memoria.
    
    Example:
        # Primera llamada carga el modelo
        manager = EmbeddingsManager("minilm")
        embedding = manager.encode_single("Hola mundo")
        
        # Segunda llamada reutiliza el mismo modelo
        manager2 = EmbeddingsManager("minilm")  # Misma instancia
    """
    
    _instance: Optional['EmbeddingsManager'] = None
    _lock = threading.Lock()
    _initialized = False
    
    def __new__(cls, model_key: str = "minilm"):
        with cls._lock:
            if cls._instance is None:
                cls._instance = super().__new__(cls)
                cls._instance._initialized = False
            return cls._instance
    
    def __init__(self, model_key: str = "minilm"):
        if self._initialized:
            return
        
        self.model_key = model_key
        model_info = EMBEDDING_MODELS.get(model_key, EMBEDDING_MODELS["minilm"])
        self.model_name = model_info["name"]
        self.expected_dim = model_info["dim"]
        self._model = None
        self._initialized = True
        logger.info(f"EmbeddingsManager inicializado con modelo: {model_key}")
    
    def preload(self) -> None:
        """
        Pre-carga el modelo en memoria.
        
        Llamar al inicio del servidor para evitar latencia en la primera consulta.
        """
        _ = self.model
    
    @property
    def model(self):
        """
        Propiedad lazy que carga el modelo solo cuando se necesita.
        
        Returns:
            SentenceTransformer: Modelo de embeddings cargado
            
        Raises:
            ImportError: Si sentence-transformers no está instalado
        """
        if self._model is None:
            try:
                from sentence_transformers import SentenceTransformer
                logger.info(f"📊 Cargando modelo de embeddings: {self.model_key}")
                start = time.time()
                self._model = SentenceTransformer(self.model_name)
                elapsed = time.time() - start
                logger.info(f"✅ Modelo cargado en {elapsed:.1f}s (permanecerá en memoria)")
            except ImportError:
                logger.error("❌ sentence-transformers no instalado")
                raise ImportError(
                    "Instalar: pip install sentence-transformers"
                )
        return self._model
    
    def encode(self, texts: List[str]) -> List[List[float]]:
        """
        Genera embeddings para una lista de textos.
        
        Args:
            texts: Lista de textos a codificar
            
        Returns:
            Lista de vectores de embeddings
        """
        embeddings = self.model.encode(texts, convert_to_numpy=True)
        return [emb.tolist() for emb in embeddings]
    
    def encode_single(self, text: str) -> List[float]:
        """
        Genera embedding para un solo texto.
        
        Args:
            text: Texto a codificar
            
        Returns:
            Vector de embedding
        """
        return self.encode([text])[0]
    
    @property
    def dimension(self) -> int:
        """Dimensión de los vectores de embedding."""
        return self.expected_dim
    
    @classmethod
    def reset(cls) -> None:
        """
        Reinicia el Singleton (útil para tests).
        
        Warning:
            Solo usar en tests o cuando se necesite cambiar de modelo.
        """
        with cls._lock:
            if cls._instance is not None:
                cls._instance._model = None
                cls._instance._initialized = False
            cls._instance = None
