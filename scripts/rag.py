#!/usr/bin/env python3
"""
RAG System - Retrieval-Augmented Generation
============================================

Sistema RAG flexible con opciones para diferentes casos de uso:

MODO SIMPLE (default):
    - Backend: archivos locales (~/.neuro-bitnet/rag/)
    - Sin búsqueda web automática
    - Sin memoria de conversaciones
    - Perfecto para proyectos pequeños

MODO AVANZADO (opcional):
    - Backend: SurrealDB (multi-usuario, escalable)
    - Búsqueda web automática cuando no hay información
    - Memoria de conversaciones
    - Separación por usuarios

Uso básico:
    python rag.py add "Python fue creado por Guido van Rossum"
    python rag.py query "¿Quién creó Python?"

Uso avanzado:
    python rag.py --user juan --backend surrealdb --auto-learn query "¿Qué es Docker?"
"""

import os
import sys
import json
import hashlib
import argparse
from pathlib import Path
from datetime import datetime
from abc import ABC, abstractmethod
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass, asdict, field

# ============================================================================
# Configuración
# ============================================================================

DEFAULT_LLM_URL = os.getenv("RAG_LLM_URL", "http://localhost:11435")
DEFAULT_SURREALDB_URL = os.getenv("RAG_SURREALDB_URL", "ws://localhost:8000/rpc")
DEFAULT_SURREALDB_USER = os.getenv("RAG_SURREALDB_USER", "root")
DEFAULT_SURREALDB_PASS = os.getenv("RAG_SURREALDB_PASS", "root")
DEFAULT_DATABASE = "neurobitnet"
DEFAULT_NAMESPACE = "rag"
DEFAULT_DATA_DIR = Path.home() / ".neuro-bitnet" / "rag"

# Modelos de embeddings con alias cortos
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
    """Documento almacenado en el RAG"""
    id: str
    content: str
    user_id: str
    source: str  # "manual", "file", "web", "conversation"
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())
    embedding: Optional[List[float]] = None

    def to_dict(self) -> Dict:
        d = asdict(self)
        d['embedding'] = None  # No serializar embedding
        return d


@dataclass
class SearchResult:
    """Resultado de búsqueda"""
    document: Document
    score: float


# ============================================================================
# Embeddings Manager
# ============================================================================

class EmbeddingsManager:
    """Gestiona la generación de embeddings con sentence-transformers"""
    
    def __init__(self, model_key: str = "minilm"):
        self.model_key = model_key
        model_info = EMBEDDING_MODELS.get(model_key, EMBEDDING_MODELS["minilm"])
        self.model_name = model_info["name"]
        self.expected_dim = model_info["dim"]
        self._model = None
    
    @property
    def model(self):
        if self._model is None:
            try:
                from sentence_transformers import SentenceTransformer
                print(f"📊 Cargando modelo de embeddings: {self.model_key}")
                self._model = SentenceTransformer(self.model_name)
            except ImportError:
                print("❌ Error: sentence-transformers no instalado")
                print("   Instalar: pip install sentence-transformers")
                sys.exit(1)
        return self._model
    
    def encode(self, texts: List[str]) -> List[List[float]]:
        """Genera embeddings para una lista de textos"""
        embeddings = self.model.encode(texts, convert_to_numpy=True)
        return [emb.tolist() for emb in embeddings]
    
    def encode_single(self, text: str) -> List[float]:
        """Genera embedding para un solo texto"""
        return self.encode([text])[0]
    
    @property
    def dimension(self) -> int:
        """Dimensión de los embeddings"""
        return self.expected_dim


# ============================================================================
# Web Search (opcional)
# ============================================================================

class WebSearcher:
    """Busca información en la web (DuckDuckGo + Wikipedia)"""
    
    def __init__(self):
        self.headers = {
            "User-Agent": "Mozilla/5.0 (compatible; neuro-bitnet RAG/1.0)"
        }
    
    def search_duckduckgo(self, query: str, max_results: int = 3) -> List[Dict]:
        """Busca usando DuckDuckGo Instant Answer API (sin API key)"""
        try:
            import requests
            from urllib.parse import quote_plus
            
            url = f"https://api.duckduckgo.com/?q={quote_plus(query)}&format=json&no_html=1"
            response = requests.get(url, headers=self.headers, timeout=10)
            data = response.json()
            
            results = []
            
            # Respuesta principal
            if data.get("Abstract"):
                results.append({
                    "title": data.get("Heading", query),
                    "content": data["Abstract"],
                    "source": data.get("AbstractURL", "DuckDuckGo"),
                    "type": "abstract"
                })
            
            # Temas relacionados
            for topic in data.get("RelatedTopics", [])[:max_results]:
                if isinstance(topic, dict) and "Text" in topic:
                    results.append({
                        "title": topic.get("FirstURL", "").split("/")[-1].replace("_", " "),
                        "content": topic["Text"],
                        "source": topic.get("FirstURL", "DuckDuckGo"),
                        "type": "related"
                    })
            
            return results[:max_results]
            
        except Exception as e:
            print(f"⚠️ Error en DuckDuckGo: {e}")
            return []
    
    def search_wikipedia(self, query: str, lang: str = "es") -> Optional[Dict]:
        """Busca en Wikipedia"""
        try:
            import requests
            from urllib.parse import quote_plus
            
            url = f"https://{lang}.wikipedia.org/api/rest_v1/page/summary/{quote_plus(query)}"
            response = requests.get(url, headers=self.headers, timeout=10)
            
            if response.status_code == 200:
                data = response.json()
                if data.get("extract"):
                    return {
                        "title": data.get("title", query),
                        "content": data["extract"],
                        "source": data.get("content_urls", {}).get("desktop", {}).get("page", "Wikipedia"),
                        "type": "wikipedia"
                    }
            return None
            
        except Exception as e:
            print(f"⚠️ Error en Wikipedia: {e}")
            return None
    
    def search(self, query: str, max_results: int = 3) -> List[Dict]:
        """Búsqueda combinada (Wikipedia primero, luego DuckDuckGo)"""
        results = []
        
        # Wikipedia primero (más confiable)
        wiki = self.search_wikipedia(query)
        if wiki:
            results.append(wiki)
        
        # Complementar con DuckDuckGo
        ddg = self.search_duckduckgo(query, max_results - len(results))
        results.extend(ddg)
        
        return results[:max_results]


# ============================================================================
# Storage Backend: Archivos (Default)
# ============================================================================

class FileBackend:
    """Backend simple usando archivos JSON + NumPy"""
    
    def __init__(self, base_path: Path = None, embedding_dim: int = 384):
        self.base_path = base_path or DEFAULT_DATA_DIR
        self.embedding_dim = embedding_dim
        self._cache = {}
    
    def _get_user_path(self, user_id: str) -> Path:
        return self.base_path / user_id
    
    def _load_user_data(self, user_id: str) -> Tuple[List[Document], Any]:
        """Carga datos del usuario"""
        if user_id in self._cache:
            return self._cache[user_id]
        
        import numpy as np
        
        user_path = self._get_user_path(user_id)
        docs_file = user_path / "documents.json"
        emb_file = user_path / "embeddings.npy"
        
        documents = []
        embeddings = None
        
        if docs_file.exists():
            with open(docs_file, 'r', encoding='utf-8') as f:
                data = json.load(f)
                documents = [Document(**d) for d in data]
        
        if emb_file.exists():
            embeddings = np.load(emb_file)
        
        self._cache[user_id] = (documents, embeddings)
        return documents, embeddings
    
    def _save_user_data(self, user_id: str, documents: List[Document], embeddings):
        """Guarda datos del usuario"""
        import numpy as np
        
        user_path = self._get_user_path(user_id)
        user_path.mkdir(parents=True, exist_ok=True)
        
        # Guardar documentos (sin embeddings)
        docs_data = [d.to_dict() for d in documents]
        with open(user_path / "documents.json", 'w', encoding='utf-8') as f:
            json.dump(docs_data, f, ensure_ascii=False, indent=2)
        
        # Guardar embeddings
        if embeddings is not None and len(embeddings) > 0:
            np.save(user_path / "embeddings.npy", np.array(embeddings))
        
        self._cache[user_id] = (documents, embeddings)
    
    def initialize(self, user_id: str) -> None:
        """Inicializa el storage para un usuario"""
        user_path = self._get_user_path(user_id)
        user_path.mkdir(parents=True, exist_ok=True)
    
    def add_document(self, doc: Document) -> str:
        """Agrega un documento"""
        import numpy as np
        
        documents, embeddings = self._load_user_data(doc.user_id)
        
        # Generar ID si no tiene
        if not doc.id:
            doc.id = hashlib.md5(
                f"{doc.content}{datetime.now().isoformat()}".encode()
            ).hexdigest()[:12]
        
        documents.append(doc)
        
        # Agregar embedding
        if embeddings is None:
            embeddings = np.array([doc.embedding])
        else:
            embeddings = np.vstack([embeddings, [doc.embedding]])
        
        self._save_user_data(doc.user_id, documents, embeddings)
        return doc.id
    
    def search(self, embedding: List[float], user_id: str, top_k: int = 3) -> List[SearchResult]:
        """Busca documentos similares usando similitud coseno"""
        import numpy as np
        
        documents, embeddings = self._load_user_data(user_id)
        
        if not documents or embeddings is None:
            return []
        
        # Similitud coseno
        query = np.array(embedding)
        similarities = np.dot(embeddings, query) / (
            np.linalg.norm(embeddings, axis=1) * np.linalg.norm(query) + 1e-10
        )
        
        # Top K
        top_indices = np.argsort(similarities)[-top_k:][::-1]
        
        results = []
        for idx in top_indices:
            if similarities[idx] > 0.1:  # Umbral mínimo
                results.append(SearchResult(
                    document=documents[idx],
                    score=float(similarities[idx])
                ))
        
        return results
    
    def list_documents(self, user_id: str, limit: int = 50) -> List[Document]:
        """Lista documentos del usuario"""
        documents, _ = self._load_user_data(user_id)
        return documents[:limit]
    
    def delete_document(self, doc_id: str, user_id: str) -> bool:
        """Elimina un documento"""
        import numpy as np
        
        documents, embeddings = self._load_user_data(user_id)
        
        for i, doc in enumerate(documents):
            if doc.id == doc_id:
                documents.pop(i)
                if embeddings is not None and len(embeddings) > i:
                    embeddings = np.delete(embeddings, i, axis=0)
                self._save_user_data(user_id, documents, embeddings)
                return True
        return False
    
    def clear_user(self, user_id: str) -> int:
        """Elimina todos los documentos del usuario"""
        documents, _ = self._load_user_data(user_id)
        count = len(documents)
        self._save_user_data(user_id, [], None)
        return count
    
    def get_stats(self, user_id: str) -> Dict:
        """Estadísticas del usuario"""
        documents, _ = self._load_user_data(user_id)
        
        stats = {"total": len(documents), "by_source": {}}
        for doc in documents:
            stats["by_source"][doc.source] = stats["by_source"].get(doc.source, 0) + 1
        
        return stats


# ============================================================================
# Storage Backend: SurrealDB (Avanzado)
# ============================================================================

class SurrealDBBackend:
    """Backend avanzado usando SurrealDB con índices vectoriales MTREE"""
    
    def __init__(self, 
                 url: str = DEFAULT_SURREALDB_URL,
                 user: str = DEFAULT_SURREALDB_USER,
                 password: str = DEFAULT_SURREALDB_PASS,
                 namespace: str = DEFAULT_NAMESPACE,
                 database: str = DEFAULT_DATABASE,
                 embedding_dim: int = 384):
        self.url = url
        self.user = user
        self.password = password
        self.namespace = namespace
        self.database = database
        self.embedding_dim = embedding_dim
        self._db = None
        self._initialized_users = set()
    
    def _get_db(self):
        """Obtiene conexión a SurrealDB"""
        if self._db is None:
            try:
                from surrealdb import Surreal
                self._db = Surreal(self.url)
            except ImportError:
                raise ImportError(
                    "surrealdb no instalado. Instalar: pip install surrealdb\n"
                    "O usar --backend files para almacenamiento local"
                )
        return self._db
    
    def _run_async(self, coro):
        """Ejecuta corrutina de forma síncrona"""
        import asyncio
        try:
            loop = asyncio.get_event_loop()
            if loop.is_running():
                # Si ya hay un loop corriendo, crear uno nuevo
                import concurrent.futures
                with concurrent.futures.ThreadPoolExecutor() as executor:
                    future = executor.submit(asyncio.run, coro)
                    return future.result()
            return loop.run_until_complete(coro)
        except RuntimeError:
            return asyncio.run(coro)
    
    async def _connect(self):
        """Conecta a SurrealDB"""
        db = self._get_db()
        await db.connect()
        await db.signin({"user": self.user, "pass": self.password})
        await db.use(self.namespace, self.database)
        return db
    
    async def _ensure_schema(self, user_id: str):
        """Crea esquema si no existe"""
        if user_id in self._initialized_users:
            return
        
        db = await self._connect()
        
        # Crear tabla con índice vectorial MTREE
        await db.query(f"""
            DEFINE TABLE IF NOT EXISTS documents SCHEMAFULL;
            DEFINE FIELD IF NOT EXISTS content ON documents TYPE string;
            DEFINE FIELD IF NOT EXISTS user_id ON documents TYPE string;
            DEFINE FIELD IF NOT EXISTS source ON documents TYPE string;
            DEFINE FIELD IF NOT EXISTS metadata ON documents TYPE object;
            DEFINE FIELD IF NOT EXISTS created_at ON documents TYPE datetime;
            DEFINE FIELD IF NOT EXISTS embedding ON documents TYPE array<float>;
            
            DEFINE INDEX IF NOT EXISTS idx_user ON documents FIELDS user_id;
            DEFINE INDEX IF NOT EXISTS idx_embedding ON documents 
                FIELDS embedding MTREE DIMENSION {self.embedding_dim} DIST COSINE;
        """)
        
        self._initialized_users.add(user_id)
    
    def initialize(self, user_id: str) -> None:
        self._run_async(self._ensure_schema(user_id))
    
    def add_document(self, doc: Document) -> str:
        async def _add():
            await self._ensure_schema(doc.user_id)
            db = await self._connect()
            
            result = await db.query("""
                CREATE documents CONTENT {
                    content: $content,
                    user_id: $user_id,
                    source: $source,
                    metadata: $metadata,
                    created_at: time::now(),
                    embedding: $embedding
                }
            """, {
                "content": doc.content,
                "user_id": doc.user_id,
                "source": doc.source,
                "metadata": doc.metadata,
                "embedding": doc.embedding
            })
            
            if result and len(result) > 0:
                res = result[0].get("result", [])
                if res:
                    return str(res[0].get("id", doc.id))
            return doc.id
        
        return self._run_async(_add())
    
    def search(self, embedding: List[float], user_id: str, top_k: int = 3) -> List[SearchResult]:
        async def _search():
            await self._ensure_schema(user_id)
            db = await self._connect()
            
            result = await db.query("""
                SELECT *, vector::similarity::cosine(embedding, $embedding) AS score
                FROM documents
                WHERE user_id = $user_id
                ORDER BY score DESC
                LIMIT $top_k
            """, {
                "embedding": embedding,
                "user_id": user_id,
                "top_k": top_k
            })
            
            results = []
            if result and len(result) > 0:
                for row in result[0].get("result", []):
                    doc = Document(
                        id=str(row.get("id", "")),
                        content=row.get("content", ""),
                        user_id=row.get("user_id", ""),
                        source=row.get("source", ""),
                        metadata=row.get("metadata", {}),
                        created_at=str(row.get("created_at", ""))
                    )
                    results.append(SearchResult(document=doc, score=row.get("score", 0)))
            
            return results
        
        return self._run_async(_search())
    
    def list_documents(self, user_id: str, limit: int = 50) -> List[Document]:
        async def _list():
            await self._ensure_schema(user_id)
            db = await self._connect()
            
            result = await db.query("""
                SELECT * FROM documents 
                WHERE user_id = $user_id 
                ORDER BY created_at DESC 
                LIMIT $limit
            """, {"user_id": user_id, "limit": limit})
            
            docs = []
            if result and len(result) > 0:
                for row in result[0].get("result", []):
                    docs.append(Document(
                        id=str(row.get("id", "")),
                        content=row.get("content", ""),
                        user_id=row.get("user_id", ""),
                        source=row.get("source", ""),
                        metadata=row.get("metadata", {}),
                        created_at=str(row.get("created_at", ""))
                    ))
            return docs
        
        return self._run_async(_list())
    
    def delete_document(self, doc_id: str, user_id: str) -> bool:
        async def _delete():
            await self._ensure_schema(user_id)
            db = await self._connect()
            await db.query(
                "DELETE FROM documents WHERE id = $id AND user_id = $user_id",
                {"id": doc_id, "user_id": user_id}
            )
            return True
        
        return self._run_async(_delete())
    
    def clear_user(self, user_id: str) -> int:
        async def _clear():
            await self._ensure_schema(user_id)
            db = await self._connect()
            
            # Contar primero
            count_result = await db.query(
                "SELECT count() FROM documents WHERE user_id = $user_id GROUP ALL",
                {"user_id": user_id}
            )
            count = 0
            if count_result and count_result[0].get("result"):
                count = count_result[0]["result"][0].get("count", 0)
            
            await db.query(
                "DELETE FROM documents WHERE user_id = $user_id",
                {"user_id": user_id}
            )
            return count
        
        return self._run_async(_clear())
    
    def get_stats(self, user_id: str) -> Dict:
        async def _stats():
            await self._ensure_schema(user_id)
            db = await self._connect()
            
            result = await db.query("""
                SELECT source, count() as count 
                FROM documents 
                WHERE user_id = $user_id 
                GROUP BY source
            """, {"user_id": user_id})
            
            stats = {"total": 0, "by_source": {}}
            if result and len(result) > 0:
                for row in result[0].get("result", []):
                    source = row.get("source", "unknown")
                    count = row.get("count", 0)
                    stats["by_source"][source] = count
                    stats["total"] += count
            
            return stats
        
        return self._run_async(_stats())


# ============================================================================
# LLM Client
# ============================================================================

class LLMClient:
    """Cliente para el servidor LLM (llama-server)"""
    
    def __init__(self, base_url: str = DEFAULT_LLM_URL):
        self.base_url = base_url.rstrip("/")
    
    def chat(self, messages: List[Dict], max_tokens: int = 512, 
             temperature: float = 0.7) -> str:
        """Envía mensaje al LLM"""
        import requests
        
        try:
            response = requests.post(
                f"{self.base_url}/v1/chat/completions",
                json={
                    "model": "bitnet",
                    "messages": messages,
                    "max_tokens": max_tokens,
                    "temperature": temperature
                },
                timeout=120
            )
            response.raise_for_status()
            return response.json()["choices"][0]["message"]["content"]
        except requests.exceptions.Timeout:
            raise ConnectionError("Timeout: el LLM tardó demasiado en responder")
        except requests.exceptions.ConnectionError:
            raise ConnectionError(f"No se pudo conectar al LLM en {self.base_url}")
        except Exception as e:
            raise ConnectionError(f"Error con el LLM: {e}")
    
    def is_available(self) -> bool:
        """Verifica si el LLM está disponible"""
        import requests
        try:
            r = requests.get(f"{self.base_url}/health", timeout=5)
            return r.status_code == 200
        except:
            return False


# ============================================================================
# RAG System Principal
# ============================================================================

class RAGSystem:
    """
    Sistema RAG flexible.
    
    Modo simple (default):
        - Backend: archivos locales
        - Sin auto-learn
        - Sin memoria de conversaciones
    
    Modo avanzado (opcional):
        - Backend: SurrealDB
        - Auto-learn desde web
        - Memoria de conversaciones
        - Multi-usuario
    """
    
    def __init__(self,
                 user_id: str = "default",
                 backend: str = "files",
                 embedding_model: str = "minilm",
                 llm_url: str = DEFAULT_LLM_URL,
                 surrealdb_url: str = DEFAULT_SURREALDB_URL,
                 auto_learn: bool = False,
                 save_conversations: bool = False):
        """
        Args:
            user_id: Identificador del usuario
            backend: "files" (simple) o "surrealdb" (avanzado)
            embedding_model: minilm, mpnet, e5, bge
            llm_url: URL del servidor LLM
            surrealdb_url: URL de SurrealDB (si se usa)
            auto_learn: Buscar en web cuando no hay info (default: False)
            save_conversations: Guardar conversaciones como documentos (default: False)
        """
        self.user_id = user_id
        self.auto_learn = auto_learn
        self.save_conversations = save_conversations
        
        # Embeddings
        self.embeddings = EmbeddingsManager(embedding_model)
        
        # LLM
        self.llm = LLMClient(llm_url)
        
        # Web searcher (solo si auto_learn está activo)
        self.web_searcher = WebSearcher() if auto_learn else None
        
        # Backend de almacenamiento
        self.backend_name = backend
        if backend == "surrealdb":
            try:
                self.storage = SurrealDBBackend(
                    url=surrealdb_url,
                    embedding_dim=self.embeddings.dimension
                )
                self.storage.initialize(user_id)
            except Exception as e:
                print(f"⚠️ SurrealDB no disponible: {e}")
                print("   Usando backend de archivos...")
                self.storage = FileBackend(embedding_dim=self.embeddings.dimension)
                self.backend_name = "files"
        else:
            self.storage = FileBackend(embedding_dim=self.embeddings.dimension)
        
        self.storage.initialize(user_id)
    
    def add(self, content: str, source: str = "manual", metadata: Dict = None) -> str:
        """Agrega un documento al RAG"""
        embedding = self.embeddings.encode_single(content)
        
        doc = Document(
            id="",
            content=content,
            user_id=self.user_id,
            source=source,
            metadata=metadata or {},
            embedding=embedding
        )
        
        return self.storage.add_document(doc)
    
    def add_file(self, file_path: str, chunk_size: int = 500) -> List[str]:
        """Agrega un archivo dividido en chunks"""
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"Archivo no encontrado: {file_path}")
        
        content = path.read_text(encoding='utf-8')
        chunks = self._split_text(content, chunk_size)
        
        doc_ids = []
        for i, chunk in enumerate(chunks):
            if chunk.strip():  # Solo chunks con contenido
                doc_id = self.add(
                    content=chunk,
                    source="file",
                    metadata={"filename": path.name, "chunk": i}
                )
                doc_ids.append(doc_id)
        
        return doc_ids
    
    def _split_text(self, text: str, chunk_size: int) -> List[str]:
        """Divide texto en chunks por párrafos"""
        paragraphs = text.split('\n\n')
        chunks = []
        current = ""
        
        for para in paragraphs:
            para = para.strip()
            if not para:
                continue
            
            if len(current) + len(para) < chunk_size:
                current += para + "\n\n"
            else:
                if current:
                    chunks.append(current.strip())
                current = para + "\n\n"
        
        if current:
            chunks.append(current.strip())
        
        return chunks
    
    def search(self, query: str, top_k: int = 3) -> List[SearchResult]:
        """Busca documentos similares"""
        embedding = self.embeddings.encode_single(query)
        return self.storage.search(embedding, self.user_id, top_k)
    
    def learn_from_web(self, query: str) -> List[str]:
        """Busca en la web y guarda lo aprendido"""
        if not self.web_searcher:
            self.web_searcher = WebSearcher()
        
        print(f"🌐 Buscando en la web: {query}")
        results = self.web_searcher.search(query)
        
        doc_ids = []
        for result in results:
            content = f"{result['title']}: {result['content']}"
            doc_id = self.add(
                content=content,
                source="web",
                metadata={
                    "query": query,
                    "url": result.get("source", ""),
                    "type": result.get("type", "")
                }
            )
            doc_ids.append(doc_id)
            print(f"  📚 Aprendido: {result['title'][:50]}...")
        
        return doc_ids
    
    def query(self, question: str, min_confidence: float = 0.5) -> str:
        """
        Consulta RAG completa:
        1. Busca en documentos locales
        2. Si auto_learn y no hay info suficiente, busca en web
        3. Genera respuesta con LLM
        """
        # 1. Buscar documentos
        results = self.search(question, top_k=3)
        
        # 2. Si no hay suficiente info y auto_learn activo
        has_good_context = any(r.score > min_confidence for r in results)
        
        if not has_good_context and self.auto_learn:
            print("🔍 Información insuficiente, buscando en la web...")
            self.learn_from_web(question)
            results = self.search(question, top_k=3)
        
        # 3. Construir contexto
        if results:
            context_parts = []
            for r in results:
                source_tag = f"[{r.document.source}]" if r.document.source else ""
                context_parts.append(f"- {r.document.content} {source_tag}")
            context = "\n".join(context_parts)
        else:
            context = "No hay información disponible sobre este tema."
        
        # 4. Verificar LLM
        if not self.llm.is_available():
            return f"⚠️ LLM no disponible.\n\nContexto encontrado:\n{context}"
        
        # 5. Generar respuesta
        messages = [
            {
                "role": "system",
                "content": """Eres un asistente con acceso a una base de conocimientos.
Responde ÚNICAMENTE usando la información del contexto.
Si no está en el contexto, di que no tienes esa información.
Sé conciso y preciso."""
            },
            {
                "role": "user",
                "content": f"""Contexto:
{context}

Pregunta: {question}

Responde basándote en el contexto:"""
            }
        ]
        
        response = self.llm.chat(messages, max_tokens=512, temperature=0.3)
        
        # 6. Guardar conversación si está activo
        if self.save_conversations:
            self.add(
                content=f"P: {question}\nR: {response}",
                source="conversation",
                metadata={"type": "qa"}
            )
        
        return response
    
    def list(self, limit: int = 50) -> List[Document]:
        """Lista documentos"""
        return self.storage.list_documents(self.user_id, limit)
    
    def delete(self, doc_id: str) -> bool:
        """Elimina un documento"""
        return self.storage.delete_document(doc_id, self.user_id)
    
    def clear(self) -> int:
        """Elimina todos los documentos"""
        return self.storage.clear_user(self.user_id)
    
    def stats(self) -> Dict:
        """Obtiene estadísticas"""
        return self.storage.get_stats(self.user_id)


# ============================================================================
# CLI
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="RAG System - Retrieval-Augmented Generation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Ejemplos (modo simple):
  %(prog)s add "Python fue creado por Guido van Rossum"
  %(prog)s query "¿Quién creó Python?"
  %(prog)s add-file documento.txt
  %(prog)s list
  %(prog)s interactive

Ejemplos (modo avanzado):
  %(prog)s --backend surrealdb --user juan add "Información personal"
  %(prog)s --auto-learn query "¿Qué es Kubernetes?"
  %(prog)s --save-conversations interactive

Variables de entorno:
  RAG_LLM_URL        URL del LLM (default: http://localhost:11435)
  RAG_SURREALDB_URL  URL de SurrealDB (default: ws://localhost:8000/rpc)
        """
    )
    
    # Opciones generales
    parser.add_argument("--user", "-u", default="default",
                        help="ID del usuario (default: default)")
    parser.add_argument("--backend", "-b", choices=["files", "surrealdb"],
                        default="files",
                        help="Backend: files (simple) o surrealdb (avanzado)")
    parser.add_argument("--embedding-model", "-e",
                        choices=list(EMBEDDING_MODELS.keys()),
                        default="minilm",
                        help="Modelo de embeddings (default: minilm)")
    parser.add_argument("--llm-url", default=DEFAULT_LLM_URL,
                        help="URL del servidor LLM")
    parser.add_argument("--surrealdb-url", default=DEFAULT_SURREALDB_URL,
                        help="URL de SurrealDB")
    
    # Opciones avanzadas
    parser.add_argument("--auto-learn", action="store_true",
                        help="Buscar en web cuando no hay información")
    parser.add_argument("--save-conversations", action="store_true",
                        help="Guardar conversaciones como documentos")
    
    # Subcomandos
    subparsers = parser.add_subparsers(dest="command", help="Comandos")
    
    # add
    add_p = subparsers.add_parser("add", help="Agregar documento")
    add_p.add_argument("content", help="Contenido del documento")
    
    # add-file
    file_p = subparsers.add_parser("add-file", help="Agregar archivo")
    file_p.add_argument("path", help="Ruta al archivo")
    file_p.add_argument("--chunk-size", type=int, default=500,
                        help="Tamaño de chunks (default: 500)")
    
    # query
    query_p = subparsers.add_parser("query", help="Consultar con RAG")
    query_p.add_argument("question", help="Pregunta")
    
    # learn
    learn_p = subparsers.add_parser("learn", help="Aprender de la web")
    learn_p.add_argument("topic", help="Tema a investigar")
    
    # search
    search_p = subparsers.add_parser("search", help="Buscar documentos")
    search_p.add_argument("query", help="Búsqueda")
    search_p.add_argument("--top-k", type=int, default=3,
                          help="Número de resultados")
    
    # list
    list_p = subparsers.add_parser("list", help="Listar documentos")
    list_p.add_argument("--limit", type=int, default=20,
                        help="Máximo de documentos")
    
    # delete
    del_p = subparsers.add_parser("delete", help="Eliminar documento")
    del_p.add_argument("doc_id", help="ID del documento")
    
    # stats
    subparsers.add_parser("stats", help="Mostrar estadísticas")
    
    # clear
    subparsers.add_parser("clear", help="Eliminar todos los documentos")
    
    # interactive
    subparsers.add_parser("interactive", help="Modo interactivo")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    # Banner
    print("🧠 RAG System")
    print(f"   Usuario: {args.user}")
    print(f"   Backend: {args.backend}")
    print(f"   Embeddings: {args.embedding_model}")
    if args.auto_learn:
        print("   Auto-learn: ✓")
    if args.save_conversations:
        print("   Save conversations: ✓")
    print()
    
    # Crear RAG
    rag = RAGSystem(
        user_id=args.user,
        backend=args.backend,
        embedding_model=args.embedding_model,
        llm_url=args.llm_url,
        surrealdb_url=args.surrealdb_url,
        auto_learn=args.auto_learn,
        save_conversations=args.save_conversations
    )
    
    print(f"   Storage: {rag.backend_name}")
    print()
    
    # Ejecutar comando
    if args.command == "add":
        doc_id = rag.add(args.content)
        print(f"✅ Documento agregado: {doc_id}")
    
    elif args.command == "add-file":
        doc_ids = rag.add_file(args.path, args.chunk_size)
        print(f"✅ Archivo agregado: {len(doc_ids)} chunks")
    
    elif args.command == "query":
        print("🤔 Procesando...\n")
        response = rag.query(args.question)
        print(f"💬 {response}")
    
    elif args.command == "learn":
        doc_ids = rag.learn_from_web(args.topic)
        print(f"\n✅ Aprendidos {len(doc_ids)} documentos")
    
    elif args.command == "search":
        results = rag.search(args.query, top_k=args.top_k)
        if results:
            print(f"🔍 {len(results)} resultados:\n")
            for i, r in enumerate(results, 1):
                print(f"{i}. [{r.score:.2f}] [{r.document.source}]")
                content = r.document.content[:150]
                if len(r.document.content) > 150:
                    content += "..."
                print(f"   {content}\n")
        else:
            print("❌ Sin resultados")
    
    elif args.command == "list":
        docs = rag.list(limit=args.limit)
        if docs:
            print(f"📚 {len(docs)} documentos:\n")
            for doc in docs:
                content = doc.content[:80]
                if len(doc.content) > 80:
                    content += "..."
                print(f"  [{doc.id}] [{doc.source}] {content}")
        else:
            print("❌ Sin documentos")
    
    elif args.command == "delete":
        if rag.delete(args.doc_id):
            print(f"✅ Documento {args.doc_id} eliminado")
        else:
            print(f"❌ Documento {args.doc_id} no encontrado")
    
    elif args.command == "stats":
        stats = rag.stats()
        print(f"📊 Estadísticas de '{args.user}':")
        print(f"   Total: {stats['total']} documentos")
        if stats.get('by_source'):
            print("   Por fuente:")
            for src, count in stats['by_source'].items():
                print(f"     - {src}: {count}")
    
    elif args.command == "clear":
        confirm = input(f"⚠️ ¿Eliminar TODOS los documentos de '{args.user}'? (s/N): ")
        if confirm.lower() == 's':
            count = rag.clear()
            print(f"✅ {count} documentos eliminados")
        else:
            print("❌ Cancelado")
    
    elif args.command == "interactive":
        print("💬 Modo interactivo")
        print("   Comandos: /add <texto>, /learn <tema>, /search <query>")
        print("             /stats, /clear, /help, salir")
        print()
        
        while True:
            try:
                user_input = input("Tú: ").strip()
            except (KeyboardInterrupt, EOFError):
                print("\n👋 ¡Hasta luego!")
                break
            
            if not user_input:
                continue
            
            if user_input.lower() in ['salir', 'exit', 'quit', 'q']:
                print("👋 ¡Hasta luego!")
                break
            
            # Comandos especiales
            if user_input.startswith("/add "):
                doc_id = rag.add(user_input[5:])
                print(f"✅ Agregado: {doc_id}\n")
                continue
            
            if user_input.startswith("/learn "):
                ids = rag.learn_from_web(user_input[7:])
                print(f"✅ Aprendidos {len(ids)} docs\n")
                continue
            
            if user_input.startswith("/search "):
                results = rag.search(user_input[8:])
                for r in results:
                    print(f"  [{r.score:.2f}] {r.document.content[:100]}...")
                print()
                continue
            
            if user_input == "/stats":
                s = rag.stats()
                print(f"📊 {s['total']} docs | {s.get('by_source', {})}\n")
                continue
            
            if user_input == "/clear":
                rag.clear()
                print("✅ Limpiado\n")
                continue
            
            if user_input == "/help":
                print("  /add <texto>   - Agregar documento")
                print("  /learn <tema>  - Aprender de la web")
                print("  /search <q>    - Buscar documentos")
                print("  /stats         - Ver estadísticas")
                print("  /clear         - Limpiar todo")
                print("  salir          - Salir\n")
                continue
            
            # Consulta normal
            print("🤔 Pensando...")
            response = rag.query(user_input)
            print(f"\n🤖 {response}\n")


if __name__ == "__main__":
    main()
