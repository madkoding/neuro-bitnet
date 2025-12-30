#!/usr/bin/env python3
"""
BitNet RAG Simple (Keyword-based)
=================================
Sistema RAG usando búsqueda por keywords (sin embeddings externos).
Ideal para BitNet ya que no requiere modelo de embeddings adicional.

Uso:
    python rag_simple.py -q "¿Cuál es la capital de Francia?"
    python rag_simple.py -i  # Modo interactivo
"""

import json
import re
import unicodedata
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

# =============================================================================
# Estructuras de datos
# =============================================================================

@dataclass
class Document:
    """Un documento en la base de conocimiento."""
    text: str
    category: str = "general"
    keywords: set = field(default_factory=set)
    metadata: dict = field(default_factory=dict)

@dataclass
class SearchResult:
    """Resultado de búsqueda."""
    document: Document
    score: float
    matching_keywords: set = field(default_factory=set)

# =============================================================================
# Base de conocimiento
# =============================================================================

class KnowledgeBase:
    """Base de conocimiento con búsqueda por keywords."""
    
    def __init__(self):
        self.documents: list[Document] = []
        self._text_index: set[str] = set()
    
    def add(self, text: str, category: str = "general", 
            extra_keywords: Optional[list[str]] = None):
        """Agrega un documento extrayendo keywords automáticamente."""
        if text in self._text_index:
            return
        
        self._text_index.add(text)
        
        # Extraer keywords del texto
        keywords = self._extract_keywords(text)
        if extra_keywords:
            keywords.update(k.lower() for k in extra_keywords)
        
        doc = Document(
            text=text,
            category=category,
            keywords=keywords
        )
        self.documents.append(doc)
    
    def _extract_keywords(self, text: str) -> set:
        """Extrae keywords relevantes de un texto."""
        # Normalizar
        text_norm = self._normalize(text)
        
        # Palabras stopwords en español
        stopwords = {
            'el', 'la', 'los', 'las', 'un', 'una', 'unos', 'unas',
            'de', 'del', 'al', 'a', 'en', 'con', 'por', 'para',
            'es', 'son', 'fue', 'ser', 'que', 'se', 'su', 'sus',
            'y', 'o', 'pero', 'como', 'más', 'mas',
            'the', 'a', 'an', 'is', 'are', 'was', 'of', 'to', 'in', 'for', 'and', 'or'
        }
        
        # Extraer palabras significativas
        words = text_norm.split()
        keywords = set()
        for word in words:
            if len(word) > 2 and word not in stopwords:
                keywords.add(word)
        
        return keywords
    
    def _normalize(self, text: str) -> str:
        """Normaliza texto removiendo acentos y puntuación."""
        # Remover acentos
        text = unicodedata.normalize('NFD', text)
        text = ''.join(c for c in text if unicodedata.category(c) != 'Mn')
        # Remover puntuación y convertir a minúsculas
        text = re.sub(r'[^\w\s]', ' ', text)
        return text.lower()
    
    def search(self, query: str, top_k: int = 3, 
               category: Optional[str] = None) -> list[SearchResult]:
        """Busca documentos relevantes por keywords."""
        if not self.documents:
            return []
        
        query_norm = self._normalize(query)
        query_keywords = self._extract_keywords(query)
        
        results = []
        for doc in self.documents:
            if category and doc.category != category:
                continue
            
            # Calcular matching
            matching = query_keywords & doc.keywords
            if not matching:
                continue
            
            # Score basado en proporción de keywords que matchean
            score = len(matching) / len(query_keywords) if query_keywords else 0
            
            # Bonus si el texto contiene la query exacta
            if query_norm in self._normalize(doc.text):
                score += 0.3
            
            # Bonus por palabras clave importantes
            important_words = {'capital', 'es', 'fue', 'significa', 'lenguaje'}
            if matching & important_words:
                score += 0.1
            
            results.append(SearchResult(
                document=doc,
                score=min(score, 1.0),
                matching_keywords=matching
            ))
        
        results.sort(key=lambda x: x.score, reverse=True)
        return results[:top_k]
    
    def save(self, filepath: str):
        """Guarda la base de conocimiento."""
        data = []
        for doc in self.documents:
            data.append({
                "text": doc.text,
                "category": doc.category,
                "keywords": list(doc.keywords),
                "metadata": doc.metadata
            })
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
    
    def load(self, filepath: str):
        """Carga la base de conocimiento."""
        with open(filepath, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        self.documents = []
        self._text_index = set()
        for item in data:
            doc = Document(
                text=item["text"],
                category=item.get("category", "general"),
                keywords=set(item.get("keywords", [])),
                metadata=item.get("metadata", {})
            )
            self.documents.append(doc)
            self._text_index.add(doc.text)

# =============================================================================
# Sistema RAG Simple
# =============================================================================

class SimpleRAG:
    """Sistema RAG usando solo keywords (sin embeddings)."""
    
    def __init__(self, base_url: str = DEFAULT_URL):
        self.base_url = base_url.rstrip('/')
        self.kb = KnowledgeBase()
        self.session = requests.Session()
        
        # System prompt con placeholder para contexto
        self.system_template = """Eres un asistente de IA preciso y útil.

INFORMACIÓN RELEVANTE:
{context}

INSTRUCCIONES:
- Usa la información proporcionada para responder
- Si la respuesta está en la información, úsala directamente
- Responde en el mismo idioma que el usuario
- Sé conciso y directo"""
    
    def add_knowledge(self, text: str, category: str = "general"):
        """Agrega conocimiento a la base."""
        self.kb.add(text, category)
    
    def search(self, query: str, top_k: int = 3) -> list[SearchResult]:
        """Busca información relevante."""
        return self.kb.search(query, top_k)
    
    def query(self, question: str, top_k: int = 3, 
              max_tokens: int = 200, temperature: float = 0.3) -> dict:
        """Responde una pregunta usando RAG."""
        
        # 1. Buscar información relevante
        results = self.search(question, top_k)
        
        # 2. Construir contexto
        if results:
            context_parts = []
            for r in results:
                context_parts.append(f"- {r.document.text}")
            context = "\n".join(context_parts)
        else:
            context = "(No se encontró información específica)"
        
        # 3. Crear system prompt con contexto
        system_content = self.system_template.format(context=context)
        
        # 4. Consultar al modelo
        try:
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
        except Exception as e:
            answer = f"Error: {e}"
        
        return {
            "answer": answer,
            "context": context,
            "sources": [r.document.text for r in results],
            "scores": [r.score for r in results]
        }
    
    def save(self, filepath: str):
        """Guarda la base de conocimiento."""
        self.kb.save(filepath)
    
    def load(self, filepath: str):
        """Carga la base de conocimiento."""
        self.kb.load(filepath)

# =============================================================================
# Base de conocimiento predefinida
# =============================================================================

DEFAULT_KNOWLEDGE = {
    "geografía": [
        "París es la capital de Francia, un país de Europa occidental.",
        "Madrid es la capital de España, un país de la península ibérica.",
        "Londres es la capital del Reino Unido.",
        "Tokio es la capital de Japón.",
        "Berlín es la capital de Alemania.",
        "Roma es la capital de Italia.",
        "Washington D.C. es la capital de Estados Unidos.",
        "Pekín o Beijing es la capital de China.",
        "Moscú es la capital de Rusia.",
        "Buenos Aires es la capital de Argentina.",
        "Ciudad de México es la capital de México.",
        "Brasilia es la capital de Brasil.",
    ],
    "ciencia": [
        "El agua tiene la fórmula química H2O, compuesta por hidrógeno y oxígeno.",
        "La velocidad de la luz es aproximadamente 299,792 kilómetros por segundo.",
        "Albert Einstein desarrolló la teoría de la relatividad.",
        "El ADN contiene la información genética de los seres vivos.",
        "La gravedad es la fuerza de atracción entre objetos con masa.",
    ],
    "tecnología": [
        "Python es un lenguaje de programación de alto nivel, interpretado y de propósito general.",
        "JavaScript es el lenguaje de programación principal para desarrollo web.",
        "Git es un sistema de control de versiones creado por Linus Torvalds.",
        "Docker es una plataforma para ejecutar aplicaciones en contenedores.",
        "Linux es un sistema operativo de código abierto basado en Unix.",
    ],
    "matemáticas": [
        "Pi (π) es aproximadamente 3.14159, la relación entre circunferencia y diámetro.",
        "El teorema de Pitágoras establece que a² + b² = c² en triángulos rectángulos.",
        "El factorial de n (n!) es el producto de todos los enteros de 1 a n.",
    ]
}

def load_default_knowledge(rag: SimpleRAG):
    """Carga la base de conocimiento predefinida."""
    print("📚 Cargando base de conocimiento...")
    for category, docs in DEFAULT_KNOWLEDGE.items():
        for text in docs:
            rag.add_knowledge(text, category)
    print(f"✅ Cargados {len(rag.kb.documents)} documentos")

# =============================================================================
# CLI
# =============================================================================

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="BitNet RAG Simple")
    parser.add_argument("--url", default=DEFAULT_URL, help="URL del servidor BitNet")
    parser.add_argument("--load", help="Cargar KB desde archivo JSON")
    parser.add_argument("--save", help="Guardar KB en archivo JSON")
    parser.add_argument("--query", "-q", help="Pregunta a responder")
    parser.add_argument("--interactive", "-i", action="store_true", help="Modo interactivo")
    parser.add_argument("--no-default", action="store_true", help="No cargar conocimiento predefinido")
    args = parser.parse_args()
    
    print("🤖 BitNet RAG Simple (Keyword-based)")
    print(f"🔗 Servidor: {args.url}")
    print()
    
    # Verificar servidor
    try:
        r = requests.get(f"{args.url}/health", timeout=5)
        if r.status_code != 200 or r.json().get("status") != "ok":
            print("❌ Servidor no disponible")
            return
        print("✅ Servidor disponible")
    except Exception as e:
        print(f"❌ No se puede conectar: {e}")
        return
    
    # Crear RAG
    rag = SimpleRAG(args.url)
    
    # Cargar conocimiento
    if args.load:
        print(f"📂 Cargando desde {args.load}...")
        rag.load(args.load)
    elif not args.no_default:
        load_default_knowledge(rag)
    
    # Guardar si se especifica
    if args.save:
        print(f"💾 Guardando en {args.save}...")
        rag.save(args.save)
    
    # Consulta única
    if args.query:
        print(f"\n❓ Pregunta: {args.query}")
        result = rag.query(args.query)
        print(f"\n💬 Respuesta: {result['answer']}")
        if result['sources']:
            print(f"\n📚 Fuentes ({len(result['sources'])}):")
            for i, (src, score) in enumerate(zip(result['sources'], result['scores']), 1):
                print(f"   {i}. [{score:.0%}] {src[:70]}...")
        return
    
    # Modo interactivo
    if args.interactive:
        print("\n🎯 Modo interactivo (escribe 'salir' para terminar)")
        print("-" * 50)
        
        while True:
            try:
                q = input("\n❓ Pregunta: ").strip()
                if q.lower() in ['salir', 'exit', 'quit', 'q']:
                    print("👋 ¡Hasta luego!")
                    break
                if not q:
                    continue
                
                result = rag.query(q)
                print(f"\n💬 Respuesta: {result['answer']}")
                if result['sources']:
                    print(f"\n📚 Fuentes:")
                    for i, (src, score) in enumerate(zip(result['sources'], result['scores']), 1):
                        print(f"   {i}. [{score:.0%}] {src[:60]}...")
                        
            except KeyboardInterrupt:
                print("\n👋 ¡Hasta luego!")
                break
    
    # Si no se especifica query ni interactivo, mostrar ayuda
    if not args.query and not args.interactive:
        print("\n💡 Uso:")
        print("   python rag_simple.py -q '¿Cuál es la capital de Francia?'")
        print("   python rag_simple.py -i  # Modo interactivo")

if __name__ == "__main__":
    main()
