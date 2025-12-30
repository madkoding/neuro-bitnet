#!/usr/bin/env python3
"""
BitNet RAG (Retrieval-Augmented Generation)
==========================================
Sistema simple de RAG usando embeddings del propio modelo BitNet.

Permite agregar conocimiento externo para mejorar las respuestas.

Uso:
    # Crear base de conocimiento
    rag = BitNetRAG()
    rag.add_document("París es la capital de Francia", category="geografía")
    rag.add_document("Madrid es la capital de España", category="geografía")
    
    # Consultar con contexto aumentado
    response = rag.query("¿Cuál es la capital de Francia?")
"""

import json
import numpy as np
from typing import Optional
from dataclasses import dataclass, field

try:
    import requests
except ImportError:
    print("❌ Instala requests: pip install requests")
    exit(1)

# =============================================================================
# Configuración
# =============================================================================

DEFAULT_URL = "http://localhost:11435"
EMBEDDING_DIM = 2048  # BitNet-2B tiene ~2048 dimensiones

# =============================================================================
# Estructuras de datos
# =============================================================================

@dataclass
class Document:
    """Un documento en la base de conocimiento."""
    text: str
    embedding: np.ndarray
    category: str = "general"
    metadata: dict = field(default_factory=dict)

@dataclass
class SearchResult:
    """Resultado de búsqueda."""
    document: Document
    score: float

# =============================================================================
# Cliente de Embeddings
# =============================================================================

class EmbeddingClient:
    """Cliente para obtener embeddings del servidor BitNet."""
    
    def __init__(self, base_url: str = DEFAULT_URL):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
    
    def get_embedding(self, text: str) -> np.ndarray:
        """Obtiene el embedding de un texto."""
        response = self.session.post(
            f"{self.base_url}/v1/embeddings",
            json={
                "input": text,
                "model": "bitnet"
            },
            timeout=30
        )
        
        if response.status_code != 200:
            raise Exception(f"Error getting embedding: {response.text}")
        
        data = response.json()
        embedding = data["data"][0]["embedding"]
        return np.array(embedding, dtype=np.float32)
    
    def get_embeddings_batch(self, texts: list[str]) -> list[np.ndarray]:
        """Obtiene embeddings de múltiples textos."""
        response = self.session.post(
            f"{self.base_url}/v1/embeddings",
            json={
                "input": texts,
                "model": "bitnet"
            },
            timeout=60
        )
        
        if response.status_code != 200:
            raise Exception(f"Error getting embeddings: {response.text}")
        
        data = response.json()
        return [np.array(item["embedding"], dtype=np.float32) for item in data["data"]]

# =============================================================================
# Base de conocimiento vectorial
# =============================================================================

class VectorStore:
    """Almacén de vectores con búsqueda híbrida (embeddings + keywords)."""
    
    def __init__(self):
        self.documents: list[Document] = []
        self._text_index: set[str] = set()  # Para evitar duplicados
    
    def add(self, document: Document):
        """Agrega un documento (evita duplicados)."""
        # Evitar duplicados basados en texto
        if document.text in self._text_index:
            return
        self._text_index.add(document.text)
        self.documents.append(document)
    
    def search(self, query_embedding: np.ndarray, query_text: str = "", 
               top_k: int = 3, category: Optional[str] = None) -> list[SearchResult]:
        """Búsqueda híbrida: embeddings + keywords."""
        if not self.documents:
            return []
        
        results = []
        # Normalizar query para matching
        query_lower = query_text.lower()
        query_words = set(self._normalize_text(query_text).split())
        
        for doc in self.documents:
            # Filtrar por categoría si se especifica
            if category and doc.category != category:
                continue
            
            # Score por embeddings (similitud coseno)
            embedding_score = self._cosine_similarity(query_embedding, doc.embedding)
            
            # Score por keywords mejorado
            keyword_score = self._keyword_score(query_lower, query_words, doc.text)
            
            # Score combinado: 30% embeddings + 70% keywords 
            # (keywords más peso porque embeddings del modelo no son ideales)
            combined_score = 0.3 * embedding_score + 0.7 * keyword_score
            
            results.append(SearchResult(document=doc, score=combined_score))
        
        # Ordenar por score descendente
        results.sort(key=lambda x: x.score, reverse=True)
        return results[:top_k]
    
    def _normalize_text(self, text: str) -> str:
        """Normaliza texto removiendo acentos y puntuación."""
        import unicodedata
        # Remover acentos
        text = unicodedata.normalize('NFD', text)
        text = ''.join(c for c in text if unicodedata.category(c) != 'Mn')
        # Remover puntuación
        text = ''.join(c if c.isalnum() or c.isspace() else ' ' for c in text)
        return text.lower()
    
    def _keyword_score(self, query_lower: str, query_words: set, doc_text: str) -> float:
        """Calcula score basado en matching de keywords."""
        doc_lower = doc_text.lower()
        doc_normalized = self._normalize_text(doc_text)
        doc_words = set(doc_normalized.split())
        
        score = 0.0
        
        # 1. Matching exacto de frases clave en la query (peso alto)
        key_phrases = ['capital de', 'capital of']
        for phrase in key_phrases:
            if phrase in query_lower and phrase in doc_lower:
                score += 0.3
        
        # 2. Matching de palabras importantes (sustantivos propios)
        important_words = [w for w in query_words if len(w) > 3]
        if important_words:
            matching = sum(1 for w in important_words if w in doc_normalized)
            score += 0.5 * (matching / len(important_words))
        
        # 3. Matching de todas las palabras
        if query_words:
            all_matching = len(query_words & doc_words)
            score += 0.2 * (all_matching / len(query_words))
        
        return min(score, 1.0)  # Normalizar a máximo 1.0
    
    def _cosine_similarity(self, a: np.ndarray, b: np.ndarray) -> float:
        """Calcula similitud coseno entre dos vectores."""
        dot_product = np.dot(a, b)
        norm_a = np.linalg.norm(a)
        norm_b = np.linalg.norm(b)
        if norm_a == 0 or norm_b == 0:
            return 0.0
        return float(dot_product / (norm_a * norm_b))
    
    def save(self, filepath: str):
        """Guarda la base de conocimiento en disco."""
        data = []
        for doc in self.documents:
            data.append({
                "text": doc.text,
                "embedding": doc.embedding.tolist(),
                "category": doc.category,
                "metadata": doc.metadata
            })
        with open(filepath, 'w') as f:
            json.dump(data, f)
    
    def load(self, filepath: str):
        """Carga la base de conocimiento desde disco."""
        with open(filepath, 'r') as f:
            data = json.load(f)
        
        self.documents = []
        for item in data:
            doc = Document(
                text=item["text"],
                embedding=np.array(item["embedding"], dtype=np.float32),
                category=item.get("category", "general"),
                metadata=item.get("metadata", {})
            )
            self.documents.append(doc)

# =============================================================================
# Sistema RAG
# =============================================================================

class BitNetRAG:
    """Sistema RAG completo para BitNet."""
    
    def __init__(self, base_url: str = DEFAULT_URL):
        self.base_url = base_url
        self.embedding_client = EmbeddingClient(base_url)
        self.vector_store = VectorStore()
        self.session = requests.Session()
        
        # System prompt base
        self.system_prompt = """Eres un asistente de IA preciso y útil.

CONTEXTO RELEVANTE:
{context}

INSTRUCCIONES:
- Usa el contexto proporcionado para responder
- Si la respuesta está en el contexto, úsala directamente
- Si no está en el contexto, responde basándote en tu conocimiento
- Responde en el mismo idioma que el usuario
- Sé conciso y preciso"""
    
    def add_document(self, text: str, category: str = "general", 
                     metadata: Optional[dict] = None):
        """Agrega un documento a la base de conocimiento."""
        embedding = self.embedding_client.get_embedding(text)
        doc = Document(
            text=text,
            embedding=embedding,
            category=category,
            metadata=metadata or {}
        )
        self.vector_store.add(doc)
        return doc
    
    def add_documents(self, texts: list[str], category: str = "general"):
        """Agrega múltiples documentos de forma eficiente."""
        embeddings = self.embedding_client.get_embeddings_batch(texts)
        for text, embedding in zip(texts, embeddings):
            doc = Document(
                text=text,
                embedding=embedding,
                category=category,
                metadata={}
            )
            self.vector_store.add(doc)
    
    def search(self, query: str, top_k: int = 3, 
               category: Optional[str] = None) -> list[SearchResult]:
        """Busca documentos relevantes para una consulta."""
        query_embedding = self.embedding_client.get_embedding(query)
        return self.vector_store.search(query_embedding, query, top_k, category)
    
    def query(self, question: str, top_k: int = 3, 
              category: Optional[str] = None,
              max_tokens: int = 200,
              temperature: float = 0.3) -> dict:
        """Responde una pregunta usando RAG."""
        
        # 1. Buscar documentos relevantes
        results = self.search(question, top_k, category)
        
        # 2. Construir contexto
        if results:
            context_parts = []
            for i, r in enumerate(results, 1):
                context_parts.append(f"{i}. {r.document.text} (relevancia: {r.score:.2f})")
            context = "\n".join(context_parts)
        else:
            context = "No se encontró contexto relevante."
        
        # 3. Construir prompt con contexto
        system_content = self.system_prompt.format(context=context)
        
        # 4. Hacer la consulta
        response = self.session.post(
            f"{self.base_url}/v1/chat/completions",
            json={
                "model": "bitnet",
                "messages": [
                    {"role": "system", "content": system_content},
                    {"role": "user", "content": question}
                ],
                "max_tokens": max_tokens,
                "temperature": temperature
            },
            timeout=60
        )
        
        data = response.json()
        answer = data.get("choices", [{}])[0].get("message", {}).get("content", "")
        
        return {
            "answer": answer,
            "context": context,
            "sources": [r.document.text for r in results],
            "scores": [r.score for r in results]
        }
    
    def save(self, filepath: str):
        """Guarda la base de conocimiento."""
        self.vector_store.save(filepath)
    
    def load(self, filepath: str):
        """Carga la base de conocimiento."""
        self.vector_store.load(filepath)

# =============================================================================
# Base de conocimiento predefinida
# =============================================================================

DEFAULT_KNOWLEDGE = [
    # Geografía - Capitales
    "París es la capital de Francia, un país de Europa occidental.",
    "Madrid es la capital de España, un país de la península ibérica.",
    "Londres es la capital del Reino Unido, formado por Inglaterra, Escocia, Gales e Irlanda del Norte.",
    "Tokio es la capital de Japón, un país insular del este de Asia.",
    "Berlín es la capital de Alemania, el país más poblado de la Unión Europea.",
    "Roma es la capital de Italia, conocida por su historia y arquitectura antigua.",
    "Washington D.C. es la capital de Estados Unidos de América.",
    "Pekín (Beijing) es la capital de China, el país más poblado del mundo.",
    "Moscú es la capital de Rusia, el país más extenso del mundo.",
    "Buenos Aires es la capital de Argentina, en América del Sur.",
    "Ciudad de México es la capital de México, en América del Norte.",
    "Brasilia es la capital de Brasil, el país más grande de Sudamérica.",
    
    # Ciencia
    "El agua tiene la fórmula química H2O, compuesta por hidrógeno y oxígeno.",
    "La velocidad de la luz es aproximadamente 299,792 kilómetros por segundo.",
    "Albert Einstein desarrolló la teoría de la relatividad en el siglo XX.",
    "El ADN contiene la información genética de los seres vivos.",
    "La gravedad es la fuerza que atrae los objetos hacia el centro de la Tierra.",
    
    # Tecnología
    "Python es un lenguaje de programación de alto nivel, interpretado y de propósito general.",
    "JavaScript es el lenguaje de programación principal para desarrollo web en navegadores.",
    "Git es un sistema de control de versiones distribuido creado por Linus Torvalds.",
    "Docker es una plataforma para desarrollar y ejecutar aplicaciones en contenedores.",
    "Linux es un sistema operativo de código abierto basado en Unix.",
    
    # Matemáticas
    "Pi (π) es aproximadamente 3.14159, la relación entre circunferencia y diámetro de un círculo.",
    "El teorema de Pitágoras establece que a² + b² = c² en triángulos rectángulos.",
    "El factorial de n (n!) es el producto de todos los enteros positivos menores o iguales a n.",
]

def create_default_knowledge(rag: BitNetRAG):
    """Carga la base de conocimiento predefinida."""
    print("📚 Cargando base de conocimiento predefinida...")
    
    # Categorizar documentos
    geografia = [d for d in DEFAULT_KNOWLEDGE if "capital" in d.lower()]
    ciencia = [d for d in DEFAULT_KNOWLEDGE if any(w in d.lower() for w in ["agua", "luz", "einstein", "adn", "gravedad"])]
    tecnologia = [d for d in DEFAULT_KNOWLEDGE if any(w in d.lower() for w in ["python", "javascript", "git", "docker", "linux"])]
    matematicas = [d for d in DEFAULT_KNOWLEDGE if any(w in d.lower() for w in ["pi", "pitágoras", "factorial"])]
    
    for text in geografia:
        rag.add_document(text, category="geografía")
    for text in ciencia:
        rag.add_document(text, category="ciencia")
    for text in tecnologia:
        rag.add_document(text, category="tecnología")
    for text in matematicas:
        rag.add_document(text, category="matemáticas")
    
    print(f"✅ Cargados {len(rag.vector_store.documents)} documentos")

# =============================================================================
# CLI
# =============================================================================

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="BitNet RAG System")
    parser.add_argument("--url", default=DEFAULT_URL, help="URL del servidor BitNet")
    parser.add_argument("--load", help="Cargar base de conocimiento desde archivo")
    parser.add_argument("--save", help="Guardar base de conocimiento en archivo")
    parser.add_argument("--query", "-q", help="Pregunta a responder")
    parser.add_argument("--interactive", "-i", action="store_true", help="Modo interactivo")
    parser.add_argument("--no-default", action="store_true", help="No cargar conocimiento predefinido")
    args = parser.parse_args()
    
    print("🤖 BitNet RAG System")
    print(f"🔗 Servidor: {args.url}")
    print()
    
    # Verificar servidor
    try:
        r = requests.get(f"{args.url}/health", timeout=5)
        if r.status_code != 200:
            print("❌ Servidor no disponible")
            return
    except:
        print("❌ No se puede conectar al servidor")
        return
    
    # Crear RAG
    rag = BitNetRAG(args.url)
    
    # Cargar conocimiento
    if args.load:
        print(f"📂 Cargando desde {args.load}...")
        rag.load(args.load)
        print(f"✅ Cargados {len(rag.vector_store.documents)} documentos")
    elif not args.no_default:
        create_default_knowledge(rag)
    
    # Guardar si se especifica
    if args.save:
        print(f"💾 Guardando en {args.save}...")
        rag.save(args.save)
        print("✅ Guardado")
    
    # Modo consulta única
    if args.query:
        print(f"\n❓ Pregunta: {args.query}")
        result = rag.query(args.query)
        print(f"\n💬 Respuesta: {result['answer']}")
        print(f"\n📚 Fuentes usadas:")
        for i, (source, score) in enumerate(zip(result['sources'], result['scores']), 1):
            print(f"   {i}. [{score:.2f}] {source[:80]}...")
        return
    
    # Modo interactivo
    if args.interactive:
        print("\n🎯 Modo interactivo (escribe 'salir' para terminar)")
        print("-" * 50)
        
        while True:
            try:
                question = input("\n❓ Tu pregunta: ").strip()
                if question.lower() in ['salir', 'exit', 'quit', 'q']:
                    print("👋 ¡Hasta luego!")
                    break
                if not question:
                    continue
                
                print("🔍 Buscando contexto relevante...")
                result = rag.query(question)
                
                print(f"\n💬 Respuesta: {result['answer']}")
                print(f"\n📚 Contexto usado:")
                for i, (source, score) in enumerate(zip(result['sources'], result['scores']), 1):
                    print(f"   {i}. [{score:.2f}] {source[:60]}...")
                    
            except KeyboardInterrupt:
                print("\n👋 ¡Hasta luego!")
                break

if __name__ == "__main__":
    main()
