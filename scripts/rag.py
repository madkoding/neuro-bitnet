#!/usr/bin/env python3
"""
BitNet RAG (Retrieval-Augmented Generation)
==========================================
Sistema RAG usando sentence-transformers para embeddings y BitNet/Falcon para generación.

Modelos de embeddings soportados:
- all-MiniLM-L6-v2     (80MB,  ~200MB RAM) - Default, rápido
- all-mpnet-base-v2    (420MB, ~500MB RAM) - Mejor calidad
- e5-large-v2          (1.2GB, ~1.5GB RAM) - Excelente calidad
- bge-large-en-v1.5    (1.3GB, ~1.5GB RAM) - Mejor multiidioma

Uso:
    # CLI interactivo
    python rag.py -i
    
    # Consulta directa
    python rag.py -q "¿Cuál es la capital de Francia?"
    
    # Agregar documentos
    python rag.py --add "París es la capital de Francia"
    python rag.py --add-file documentos.txt
    
    # Cambiar modelo de embeddings
    python rag.py -i --embedding-model all-mpnet-base-v2
    
    # Ver estadísticas
    python rag.py --stats
"""

import os
import json
import hashlib
import argparse
from pathlib import Path
from datetime import datetime
from dataclasses import dataclass, field, asdict
from typing import Optional, List, Dict, Any

try:
    import requests
except ImportError:
    print("❌ Instala requests: pip install requests")
    exit(1)

try:
    import numpy as np
except ImportError:
    print("❌ Instala numpy: pip install numpy")
    exit(1)

# =============================================================================
# Configuración
# =============================================================================

DEFAULT_URL = "http://localhost:11435"
DEFAULT_EMBEDDING_MODEL = "all-MiniLM-L6-v2"
DEFAULT_DATA_DIR = Path.home() / ".neuro-bitnet" / "rag"

# Modelos soportados con sus dimensiones
EMBEDDING_MODELS = {
    "all-MiniLM-L6-v2": {
        "dim": 384,
        "description": "Rápido y ligero (80MB)",
        "max_seq_length": 256,
    },
    "all-mpnet-base-v2": {
        "dim": 768,
        "description": "Mejor calidad (420MB)",
        "max_seq_length": 384,
    },
    "e5-large-v2": {
        "dim": 1024,
        "description": "Excelente calidad (1.2GB)",
        "max_seq_length": 512,
    },
    "bge-large-en-v1.5": {
        "dim": 1024,
        "description": "Mejor multiidioma (1.3GB)",
        "max_seq_length": 512,
    },
}

# =============================================================================
# Estructuras de datos
# =============================================================================

@dataclass
class Document:
    """Un documento en la base de conocimiento."""
    id: str
    text: str
    category: str = "general"
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())

@dataclass
class SearchResult:
    """Resultado de búsqueda."""
    document: Document
    score: float

# =============================================================================
# Embedding Client (sentence-transformers)
# =============================================================================

class EmbeddingClient:
    """Cliente para generar embeddings con sentence-transformers."""
    
    def __init__(self, model_name: str = DEFAULT_EMBEDDING_MODEL):
        self.model_name = model_name
        self.model = None
        self._load_model()
    
    def _load_model(self):
        """Carga el modelo de embeddings."""
        try:
            from sentence_transformers import SentenceTransformer
        except ImportError:
            print("❌ Instala sentence-transformers: pip install sentence-transformers")
            exit(1)
        
        print(f"📦 Cargando modelo de embeddings: {self.model_name}...")
        
        # Para e5, necesita prefijo especial
        if "e5" in self.model_name.lower():
            self.prefix = "query: "
        else:
            self.prefix = ""
        
        self.model = SentenceTransformer(self.model_name)
        self.dim = EMBEDDING_MODELS.get(self.model_name, {}).get("dim", 384)
        print(f"✅ Modelo cargado (dim={self.dim})")
    
    def get_embedding(self, text: str) -> np.ndarray:
        """Genera embedding para un texto."""
        text_with_prefix = self.prefix + text if self.prefix else text
        return self.model.encode(text_with_prefix, convert_to_numpy=True)
    
    def get_embeddings_batch(self, texts: List[str]) -> np.ndarray:
        """Genera embeddings para múltiples textos."""
        texts_with_prefix = [self.prefix + t if self.prefix else t for t in texts]
        return self.model.encode(texts_with_prefix, convert_to_numpy=True)

# =============================================================================
# Vector Store (persistente en disco)
# =============================================================================

class VectorStore:
    """Almacén de vectores con persistencia en disco."""
    
    def __init__(self, data_dir: Path, embedding_dim: int):
        self.data_dir = Path(data_dir)
        self.data_dir.mkdir(parents=True, exist_ok=True)
        self.embedding_dim = embedding_dim
        
        self.documents_file = self.data_dir / "documents.json"
        self.embeddings_file = self.data_dir / "embeddings.npy"
        
        self.documents: List[Document] = []
        self.embeddings: Optional[np.ndarray] = None
        
        self._load()
    
    def _load(self):
        """Carga documentos y embeddings desde disco."""
        if self.documents_file.exists():
            with open(self.documents_file, 'r') as f:
                data = json.load(f)
                self.documents = [Document(**doc) for doc in data]
        
        if self.embeddings_file.exists():
            self.embeddings = np.load(self.embeddings_file)
    
    def _save(self):
        """Guarda documentos y embeddings a disco."""
        with open(self.documents_file, 'w') as f:
            json.dump([asdict(doc) for doc in self.documents], f, indent=2)
        
        if self.embeddings is not None:
            np.save(self.embeddings_file, self.embeddings)
    
    def add(self, document: Document, embedding: np.ndarray):
        """Agrega un documento con su embedding."""
        self.documents.append(document)
        
        if self.embeddings is None:
            self.embeddings = embedding.reshape(1, -1)
        else:
            self.embeddings = np.vstack([self.embeddings, embedding])
        
        self._save()
    
    def search(self, query_embedding: np.ndarray, top_k: int = 5) -> List[SearchResult]:
        """Busca documentos similares por cosine similarity."""
        if self.embeddings is None or len(self.documents) == 0:
            return []
        
        # Normalizar para cosine similarity
        query_norm = query_embedding / np.linalg.norm(query_embedding)
        embeddings_norm = self.embeddings / np.linalg.norm(self.embeddings, axis=1, keepdims=True)
        
        # Calcular similitud
        similarities = np.dot(embeddings_norm, query_norm)
        
        # Obtener top_k
        top_indices = np.argsort(similarities)[::-1][:top_k]
        
        results = []
        for idx in top_indices:
            results.append(SearchResult(
                document=self.documents[idx],
                score=float(similarities[idx])
            ))
        
        return results
    
    def delete(self, doc_id: str) -> bool:
        """Elimina un documento por ID."""
        for i, doc in enumerate(self.documents):
            if doc.id == doc_id:
                self.documents.pop(i)
                if self.embeddings is not None:
                    self.embeddings = np.delete(self.embeddings, i, axis=0)
                    if len(self.embeddings) == 0:
                        self.embeddings = None
                self._save()
                return True
        return False
    
    def clear(self):
        """Elimina todos los documentos."""
        self.documents = []
        self.embeddings = None
        self._save()
    
    def stats(self) -> Dict[str, Any]:
        """Retorna estadísticas de la base de datos."""
        categories = {}
        for doc in self.documents:
            categories[doc.category] = categories.get(doc.category, 0) + 1
        
        return {
            "total_documents": len(self.documents),
            "embedding_dim": self.embedding_dim,
            "categories": categories,
            "storage_size_mb": self._get_storage_size(),
        }
    
    def _get_storage_size(self) -> float:
        """Calcula el tamaño en MB del almacenamiento."""
        size = 0
        if self.documents_file.exists():
            size += self.documents_file.stat().st_size
        if self.embeddings_file.exists():
            size += self.embeddings_file.stat().st_size
        return round(size / (1024 * 1024), 2)

# =============================================================================
# BitNet/Falcon Client (para generación)
# =============================================================================

class LLMClient:
    """Cliente para el servidor BitNet/Falcon."""
    
    def __init__(self, base_url: str = DEFAULT_URL):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
    
    def health_check(self) -> bool:
        """Verifica si el servidor está disponible."""
        try:
            r = self.session.get(f"{self.base_url}/health", timeout=5)
            return r.status_code == 200
        except:
            return False
    
    def chat(self, messages: List[Dict], max_tokens: int = 500, temperature: float = 0.7) -> str:
        """Genera una respuesta del modelo."""
        response = self.session.post(
            f"{self.base_url}/v1/chat/completions",
            json={
                "model": "bitnet",
                "messages": messages,
                "max_tokens": max_tokens,
                "temperature": temperature,
            },
            timeout=120
        )
        
        if response.status_code != 200:
            raise Exception(f"Error: {response.text}")
        
        data = response.json()
        return data["choices"][0]["message"]["content"]

# =============================================================================
# RAG System
# =============================================================================

class BitNetRAG:
    """Sistema RAG completo."""
    
    def __init__(
        self,
        llm_url: str = DEFAULT_URL,
        embedding_model: str = DEFAULT_EMBEDDING_MODEL,
        data_dir: Optional[Path] = None,
    ):
        self.llm_url = llm_url
        self.embedding_model_name = embedding_model
        
        # Validar modelo de embeddings
        if embedding_model not in EMBEDDING_MODELS:
            available = ", ".join(EMBEDDING_MODELS.keys())
            raise ValueError(f"Modelo no soportado: {embedding_model}. Disponibles: {available}")
        
        # Directorio de datos (incluye nombre del modelo para separar)
        if data_dir is None:
            data_dir = DEFAULT_DATA_DIR / embedding_model.replace("/", "_")
        self.data_dir = Path(data_dir)
        
        # Inicializar componentes
        self.embedding_client = EmbeddingClient(embedding_model)
        self.vector_store = VectorStore(self.data_dir, self.embedding_client.dim)
        self.llm_client = LLMClient(llm_url)
        
        print(f"📁 Base de datos: {self.data_dir}")
        print(f"📊 Documentos cargados: {len(self.vector_store.documents)}")
    
    def add_document(self, text: str, category: str = "general", metadata: Dict = None) -> str:
        """Agrega un documento a la base de conocimiento."""
        # Generar ID único
        doc_id = hashlib.md5(text.encode()).hexdigest()[:12]
        
        # Verificar si ya existe
        for doc in self.vector_store.documents:
            if doc.id == doc_id:
                print(f"⚠️  Documento ya existe: {doc_id}")
                return doc_id
        
        # Crear documento
        document = Document(
            id=doc_id,
            text=text,
            category=category,
            metadata=metadata or {},
        )
        
        # Generar embedding
        embedding = self.embedding_client.get_embedding(text)
        
        # Guardar
        self.vector_store.add(document, embedding)
        
        print(f"✅ Documento agregado: {doc_id} ({category})")
        return doc_id
    
    def add_documents_from_file(self, file_path: str, category: str = "general") -> int:
        """Agrega documentos desde un archivo (uno por línea)."""
        path = Path(file_path)
        if not path.exists():
            raise FileNotFoundError(f"Archivo no encontrado: {file_path}")
        
        count = 0
        with open(path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#'):
                    self.add_document(line, category=category)
                    count += 1
        
        print(f"📄 {count} documentos agregados desde {file_path}")
        return count
    
    def search(self, query: str, top_k: int = 3) -> List[SearchResult]:
        """Busca documentos relevantes."""
        query_embedding = self.embedding_client.get_embedding(query)
        return self.vector_store.search(query_embedding, top_k=top_k)
    
    def query(
        self,
        question: str,
        top_k: int = 3,
        max_tokens: int = 500,
        temperature: float = 0.7,
        show_context: bool = False,
    ) -> str:
        """Responde una pregunta usando RAG."""
        
        # Buscar contexto relevante
        results = self.search(question, top_k=top_k)
        
        if not results:
            context = "No hay información relevante en la base de conocimiento."
        else:
            context_parts = []
            for i, r in enumerate(results, 1):
                context_parts.append(f"[{i}] {r.document.text}")
            context = "\n".join(context_parts)
        
        if show_context:
            print("\n📚 Contexto encontrado:")
            for r in results:
                print(f"  [{r.score:.3f}] {r.document.text[:100]}...")
            print()
        
        # Construir prompt con contexto
        system_prompt = """Eres un asistente útil. Usa el contexto proporcionado para responder.
Si la información no está en el contexto, indica que no tienes esa información.
Responde en el mismo idioma que la pregunta."""

        user_prompt = f"""Contexto:
{context}

Pregunta: {question}

Respuesta:"""
        
        # Generar respuesta
        messages = [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt},
        ]
        
        return self.llm_client.chat(messages, max_tokens=max_tokens, temperature=temperature)
    
    def delete_document(self, doc_id: str) -> bool:
        """Elimina un documento."""
        return self.vector_store.delete(doc_id)
    
    def clear_all(self):
        """Elimina todos los documentos."""
        self.vector_store.clear()
        print("🗑️  Base de conocimiento limpiada")
    
    def stats(self) -> Dict[str, Any]:
        """Retorna estadísticas del sistema."""
        stats = self.vector_store.stats()
        stats["embedding_model"] = self.embedding_model_name
        stats["llm_url"] = self.llm_url
        return stats
    
    def list_documents(self, category: Optional[str] = None, limit: int = 20) -> List[Document]:
        """Lista documentos en la base de conocimiento."""
        docs = self.vector_store.documents
        if category:
            docs = [d for d in docs if d.category == category]
        return docs[:limit]

# =============================================================================
# CLI
# =============================================================================

def interactive_mode(rag: BitNetRAG):
    """Modo interactivo."""
    print("\n" + "=" * 60)
    print("🤖 BitNet RAG - Modo Interactivo")
    print("=" * 60)
    print("Comandos:")
    print("  /add <texto>     - Agregar documento")
    print("  /search <query>  - Buscar sin generar respuesta")
    print("  /list [cat]      - Listar documentos")
    print("  /stats           - Ver estadísticas")
    print("  /clear           - Limpiar base de conocimiento")
    print("  /context         - Toggle mostrar contexto")
    print("  /help            - Mostrar ayuda")
    print("  /quit            - Salir")
    print("=" * 60)
    print()
    
    show_context = False
    
    while True:
        try:
            user_input = input("📝 Tú: ").strip()
        except (KeyboardInterrupt, EOFError):
            print("\n👋 ¡Adiós!")
            break
        
        if not user_input:
            continue
        
        # Comandos
        if user_input.startswith('/'):
            parts = user_input.split(' ', 1)
            cmd = parts[0].lower()
            arg = parts[1] if len(parts) > 1 else ""
            
            if cmd == '/quit' or cmd == '/exit':
                print("👋 ¡Adiós!")
                break
            
            elif cmd == '/add':
                if arg:
                    rag.add_document(arg)
                else:
                    print("❌ Uso: /add <texto>")
            
            elif cmd == '/search':
                if arg:
                    results = rag.search(arg)
                    print("\n🔍 Resultados:")
                    for r in results:
                        print(f"  [{r.score:.3f}] {r.document.text[:80]}...")
                    print()
                else:
                    print("❌ Uso: /search <query>")
            
            elif cmd == '/list':
                docs = rag.list_documents(category=arg if arg else None)
                print(f"\n📋 Documentos ({len(docs)}):")
                for doc in docs:
                    print(f"  [{doc.id}] ({doc.category}) {doc.text[:60]}...")
                print()
            
            elif cmd == '/stats':
                stats = rag.stats()
                print("\n📊 Estadísticas:")
                print(f"  Modelo embeddings: {stats['embedding_model']}")
                print(f"  Documentos: {stats['total_documents']}")
                print(f"  Dimensión: {stats['embedding_dim']}")
                print(f"  Tamaño: {stats['storage_size_mb']} MB")
                print(f"  Categorías: {stats['categories']}")
                print()
            
            elif cmd == '/clear':
                confirm = input("⚠️  ¿Eliminar todos los documentos? (s/N): ")
                if confirm.lower() == 's':
                    rag.clear_all()
            
            elif cmd == '/context':
                show_context = not show_context
                print(f"📚 Mostrar contexto: {'ON' if show_context else 'OFF'}")
            
            elif cmd == '/help':
                print("\nComandos disponibles:")
                print("  /add <texto>     - Agregar documento")
                print("  /search <query>  - Buscar sin generar respuesta")
                print("  /list [cat]      - Listar documentos")
                print("  /stats           - Ver estadísticas")
                print("  /clear           - Limpiar base de conocimiento")
                print("  /context         - Toggle mostrar contexto")
                print("  /quit            - Salir")
                print()
            
            else:
                print(f"❌ Comando desconocido: {cmd}")
        
        else:
            # Consulta RAG
            try:
                response = rag.query(user_input, show_context=show_context)
                print(f"\n🤖 Asistente: {response}\n")
            except Exception as e:
                print(f"❌ Error: {e}\n")

def main():
    parser = argparse.ArgumentParser(
        description="BitNet RAG - Retrieval-Augmented Generation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Modelos de embeddings disponibles:
  all-MiniLM-L6-v2    (default) Rápido y ligero (80MB)
  all-mpnet-base-v2             Mejor calidad (420MB)
  e5-large-v2                   Excelente calidad (1.2GB)
  bge-large-en-v1.5             Mejor multiidioma (1.3GB)

Ejemplos:
  python rag.py -i                           # Modo interactivo
  python rag.py -q "¿Qué es Python?"         # Consulta directa
  python rag.py --add "Python es un lenguaje"  # Agregar documento
  python rag.py --embedding-model all-mpnet-base-v2 -i  # Usar modelo más grande
        """
    )
    
    parser.add_argument("-i", "--interactive", action="store_true",
                        help="Modo interactivo")
    parser.add_argument("-q", "--query", type=str,
                        help="Consulta directa")
    parser.add_argument("--add", type=str,
                        help="Agregar documento")
    parser.add_argument("--add-file", type=str,
                        help="Agregar documentos desde archivo")
    parser.add_argument("--category", type=str, default="general",
                        help="Categoría para documentos (default: general)")
    parser.add_argument("--stats", action="store_true",
                        help="Mostrar estadísticas")
    parser.add_argument("--list", action="store_true",
                        help="Listar documentos")
    parser.add_argument("--clear", action="store_true",
                        help="Limpiar base de conocimiento")
    parser.add_argument("--url", type=str, default=DEFAULT_URL,
                        help=f"URL del servidor LLM (default: {DEFAULT_URL})")
    parser.add_argument("--embedding-model", "-e", type=str, default=DEFAULT_EMBEDDING_MODEL,
                        choices=list(EMBEDDING_MODELS.keys()),
                        help=f"Modelo de embeddings (default: {DEFAULT_EMBEDDING_MODEL})")
    parser.add_argument("--show-context", "-c", action="store_true",
                        help="Mostrar contexto encontrado")
    
    args = parser.parse_args()
    
    print("\n🧠 BitNet RAG")
    print("=" * 60)
    
    # Inicializar RAG
    try:
        rag = BitNetRAG(
            llm_url=args.url,
            embedding_model=args.embedding_model,
        )
    except Exception as e:
        print(f"❌ Error inicializando RAG: {e}")
        exit(1)
    
    # Verificar servidor LLM
    if not rag.llm_client.health_check():
        print(f"⚠️  Servidor LLM no disponible en {args.url}")
        print("   Las consultas RAG no funcionarán, pero puedes agregar documentos.")
    else:
        print(f"✅ Servidor LLM disponible")
    
    # Ejecutar acción
    if args.stats:
        stats = rag.stats()
        print("\n📊 Estadísticas:")
        for k, v in stats.items():
            print(f"  {k}: {v}")
    
    elif args.list:
        docs = rag.list_documents()
        print(f"\n📋 Documentos ({len(docs)}):")
        for doc in docs:
            print(f"  [{doc.id}] ({doc.category}) {doc.text[:60]}...")
    
    elif args.clear:
        confirm = input("⚠️  ¿Eliminar todos los documentos? (s/N): ")
        if confirm.lower() == 's':
            rag.clear_all()
    
    elif args.add:
        rag.add_document(args.add, category=args.category)
    
    elif args.add_file:
        rag.add_documents_from_file(args.add_file, category=args.category)
    
    elif args.query:
        response = rag.query(args.query, show_context=args.show_context)
        print(f"\n🤖 Respuesta:\n{response}\n")
    
    elif args.interactive:
        interactive_mode(rag)
    
    else:
        parser.print_help()

if __name__ == "__main__":
    main()
