#!/usr/bin/env python3
"""
RAG Server - Servidor HTTP Persistente con Router Inteligente
=============================================================

Servidor HTTP que mantiene el modelo de embeddings en memoria y decide
inteligentemente cuándo usar:
- Respuesta directa del LLM (matemáticas, código, razonamiento)
- RAG local (conocimiento almacenado)
- Búsqueda web (conocimiento factual externo)

Uso:
    python -m src.server.rag_server                    # Puerto 11436
    python -m src.server.rag_server --port 8080        # Puerto personalizado
    python -m src.server.rag_server --embedding-model mpnet

API Endpoints:
    POST /query         - Consulta inteligente (auto-clasifica)
    POST /add           - Agregar documento
    POST /search        - Buscar en RAG
    POST /classify      - Solo clasificar (sin ejecutar)
    GET  /health        - Estado del servidor
    GET  /stats         - Estadísticas
    GET  /documents     - Listar documentos
"""

import sys
import time
import logging
import argparse
from datetime import datetime
from typing import Dict, Any, List, Optional, Tuple

from src.rag.core import (
    Document, 
    SearchResult, 
    EMBEDDING_MODELS, 
    DEFAULT_LLM_URL,
    DEFAULT_RAG_PORT
)
from src.rag.classifier import (
    QueryClassifier, 
    QueryCategory, 
    QueryStrategy,
    ClassificationResult
)
from src.rag.embeddings import EmbeddingsManager
from src.rag.storage import FileStorage
from src.rag.web_search import WebSearcher

# Configurar logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


# ============================================================================
# LLM Client
# ============================================================================

class LLMClient:
    """Cliente para comunicarse con el LLM."""
    
    def __init__(self, base_url: str = DEFAULT_LLM_URL):
        self.base_url = base_url.rstrip('/')
    
    def chat(
        self, 
        messages: List[Dict], 
        max_tokens: int = 512, 
        temperature: float = 0.7
    ) -> str:
        """Envía mensaje al LLM."""
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
        except Exception as e:
            raise ConnectionError(f"Error con el LLM: {e}")
    
    def is_available(self) -> bool:
        """Verifica si el LLM está disponible."""
        import requests
        try:
            r = requests.get(f"{self.base_url}/health", timeout=5)
            return r.status_code == 200
        except Exception:
            return False


# ============================================================================
# Smart RAG System
# ============================================================================

class SmartRAG:
    """
    Sistema RAG inteligente con clasificación automática.
    
    Coordina todos los componentes:
    - Clasificador de consultas
    - Embeddings
    - Storage
    - LLM Client
    - Web Searcher
    """
    
    # System prompts por categoría
    SYSTEM_PROMPTS = {
        QueryCategory.MATH: """Eres una calculadora. Responde SOLO con el número resultante.
No expliques, no muestres pasos. Solo el número.""",
        
        QueryCategory.CODE: """Eres un programador experto. Responde SOLO con código.
No expliques, no agregues comentarios innecesarios. Solo código funcional.""",
        
        QueryCategory.TOOLS: """Eres un asistente con acceso a herramientas.

HERRAMIENTAS:
- get_weather(location): Clima de una ciudad
- calculate(expression): Calcular expresión matemática
- translate(text, to_language): Traducir texto

RESPONDE SOLO CON JSON: {"tool": "nombre", "arguments": {"param": "valor"}}""",
        
        QueryCategory.REASONING: """Eres un asistente de razonamiento lógico.
Responde SOLO con la conclusión, sin repetir el problema.
Sé breve y directo.""",
    }
    
    def __init__(
        self, 
        embedding_model: str = "minilm",
        llm_url: str = DEFAULT_LLM_URL
    ):
        self.classifier = QueryClassifier()
        self.embeddings = EmbeddingsManager(embedding_model)
        self.storage = FileStorage()
        self.llm = LLMClient(llm_url)
        self.web_searcher = WebSearcher()
        
        # Pre-cargar modelo de embeddings
        logger.info("Pre-cargando modelo de embeddings...")
        self.embeddings.preload()
        
        # Estadísticas
        self.stats = {
            "queries_total": 0,
            "queries_by_category": {},
            "queries_by_strategy": {},
            "cache_hits": 0,
            "web_searches": 0,
            "start_time": datetime.now().isoformat()
        }
    
    def query(
        self, 
        question: str, 
        user_id: str = "default", 
        force_strategy: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        Procesa una consulta de forma inteligente.
        
        Args:
            question: La pregunta del usuario
            user_id: ID del usuario para RAG personalizado
            force_strategy: Forzar estrategia específica
        
        Returns:
            Diccionario con answer, classification, sources, timing
        """
        start_time = time.time()
        self.stats["queries_total"] += 1
        
        # 1. Clasificar consulta
        classification = self.classifier.classify(question)
        cat_name = classification.category.value
        self.stats["queries_by_category"][cat_name] = \
            self.stats["queries_by_category"].get(cat_name, 0) + 1
        
        # 2. Determinar estrategia
        if force_strategy:
            strategy = QueryStrategy(force_strategy)
        else:
            strategy = classification.strategy
        
        strat_name = strategy.value
        self.stats["queries_by_strategy"][strat_name] = \
            self.stats["queries_by_strategy"].get(strat_name, 0) + 1
        
        # 3. Ejecutar estrategia
        sources: List[Any] = []
        context = ""
        
        if strategy == QueryStrategy.LLM_DIRECT:
            answer = self._query_llm_direct(question, classification.category)
            
        elif strategy == QueryStrategy.RAG_LOCAL:
            context, sources = self._search_rag(question, user_id)
            answer = self._query_llm_with_context(question, context) if context else \
                     self._query_llm_direct(question)
            
        elif strategy == QueryStrategy.RAG_THEN_WEB:
            context, sources = self._search_rag(question, user_id)
            
            # Si no hay suficiente info en RAG, buscar en web
            if not sources or (sources and sources[0].score < 0.5):
                web_results = self.web_searcher.search(question)
                if web_results:
                    for result in web_results:
                        context += f"\n\n{result['title']}: {result['content']}"
                        sources.append({
                            "source": "web",
                            "title": result['title'],
                            "score": 0.8
                        })
                    self.stats["web_searches"] += 1
            
            answer = self._query_llm_with_context(question, context) if context else \
                     self._query_llm_direct(question)
            
        elif strategy == QueryStrategy.WEB_SEARCH:
            web_results = self.web_searcher.search(question)
            if web_results:
                for result in web_results:
                    context += f"\n\n{result['title']}: {result['content']}"
                    sources.append({
                        "source": "web",
                        "title": result['title'],
                        "score": 0.8
                    })
                self.stats["web_searches"] += 1
            answer = self._query_llm_with_context(question, context) if context else \
                     self._query_llm_direct(question)
        else:
            answer = self._query_llm_direct(question)
        
        elapsed = time.time() - start_time
        
        # Serializar sources
        serializable_sources = []
        for s in sources:
            if isinstance(s, SearchResult):
                serializable_sources.append(s.to_dict())
            elif isinstance(s, dict):
                serializable_sources.append(s)
        
        return {
            "answer": answer,
            "classification": classification.to_dict(),
            "sources": serializable_sources,
            "timing": {"total_ms": int(elapsed * 1000)}
        }
    
    def _query_llm_direct(
        self, 
        question: str, 
        category: Optional[QueryCategory] = None
    ) -> str:
        """Consulta directa al LLM con system prompt apropiado."""
        messages = []
        
        if category and category in self.SYSTEM_PROMPTS:
            messages.append({
                "role": "system", 
                "content": self.SYSTEM_PROMPTS[category]
            })
        
        messages.append({"role": "user", "content": question})
        return self.llm.chat(messages)
    
    def _query_llm_with_context(self, question: str, context: str) -> str:
        """Consulta al LLM con contexto RAG."""
        system_prompt = f"""Eres un asistente inteligente. Usa la siguiente información para responder:

CONTEXTO:
{context}

INSTRUCCIONES:
- Responde basándote en el contexto proporcionado
- Si el contexto no contiene la información, indícalo claramente
- Sé conciso y directo"""
        
        messages = [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": question}
        ]
        return self.llm.chat(messages)
    
    def _search_rag(
        self, 
        query: str, 
        user_id: str
    ) -> Tuple[str, List[SearchResult]]:
        """Busca en el RAG local."""
        embedding = self.embeddings.encode_single(query)
        results = self.storage.search(embedding, user_id, top_k=3)
        
        context = ""
        for r in results:
            context += f"\n- {r.document.content}"
        
        return context.strip(), results
    
    def add_document(
        self, 
        content: str, 
        user_id: str = "default", 
        source: str = "manual",
        metadata: Optional[Dict] = None
    ) -> str:
        """Agrega un documento al RAG."""
        embedding = self.embeddings.encode_single(content)
        
        doc = Document(
            id="",
            content=content,
            user_id=user_id,
            source=source,
            metadata=metadata or {},
            embedding=embedding
        )
        
        self.storage.initialize(user_id)
        return self.storage.add_document(doc)
    
    def search_documents(
        self, 
        query: str, 
        user_id: str = "default", 
        top_k: int = 5
    ) -> List[Dict]:
        """Busca documentos similares."""
        embedding = self.embeddings.encode_single(query)
        results = self.storage.search(embedding, user_id, top_k)
        return [r.to_dict() for r in results]
    
    def get_stats(self) -> Dict[str, Any]:
        """Retorna estadísticas del servidor."""
        return {
            **self.stats,
            "embedding_model": self.embeddings.model_key,
            "embedding_dim": self.embeddings.dimension,
            "llm_available": self.llm.is_available()
        }


# ============================================================================
# Flask App Factory
# ============================================================================

def create_app(rag: SmartRAG):
    """Crea la aplicación Flask."""
    try:
        from flask import Flask, request, jsonify
    except ImportError:
        logger.error("Flask no instalado. Instalar: pip install flask")
        sys.exit(1)
    
    app = Flask(__name__)
    
    @app.route('/health', methods=['GET'])
    def health():
        return jsonify({
            "status": "healthy",
            "embedding_model_loaded": rag.embeddings._model is not None,
            "llm_available": rag.llm.is_available()
        })
    
    @app.route('/stats', methods=['GET'])
    def stats():
        return jsonify(rag.get_stats())
    
    @app.route('/query', methods=['POST'])
    def query():
        data = request.get_json()
        if not data or 'question' not in data:
            return jsonify({"error": "Falta campo 'question'"}), 400
        
        try:
            result = rag.query(
                data['question'],
                data.get('user_id', 'default'),
                data.get('strategy')
            )
            return jsonify(result)
        except Exception as e:
            logger.error(f"Error en query: {e}")
            return jsonify({"error": str(e)}), 500
    
    @app.route('/classify', methods=['POST'])
    def classify():
        data = request.get_json()
        if not data or 'question' not in data:
            return jsonify({"error": "Falta campo 'question'"}), 400
        
        classification = rag.classifier.classify(data['question'])
        return jsonify(classification.to_dict())
    
    @app.route('/add', methods=['POST'])
    def add_document():
        data = request.get_json()
        if not data or 'content' not in data:
            return jsonify({"error": "Falta campo 'content'"}), 400
        
        try:
            doc_id = rag.add_document(
                data['content'],
                data.get('user_id', 'default'),
                data.get('source', 'manual'),
                data.get('metadata')
            )
            return jsonify({"id": doc_id, "status": "added"})
        except Exception as e:
            logger.error(f"Error añadiendo documento: {e}")
            return jsonify({"error": str(e)}), 500
    
    @app.route('/search', methods=['POST'])
    def search():
        data = request.get_json()
        if not data or 'query' not in data:
            return jsonify({"error": "Falta campo 'query'"}), 400
        
        try:
            results = rag.search_documents(
                data['query'],
                data.get('user_id', 'default'),
                data.get('top_k', 5)
            )
            return jsonify({"results": results})
        except Exception as e:
            logger.error(f"Error en búsqueda: {e}")
            return jsonify({"error": str(e)}), 500
    
    @app.route('/documents', methods=['GET'])
    def list_documents():
        user_id = request.args.get('user_id', 'default')
        limit = int(request.args.get('limit', 50))
        
        docs = rag.storage.list_documents(user_id, limit)
        return jsonify({
            "documents": [{
                "id": d.id,
                "content": d.content[:200] + "..." if len(d.content) > 200 else d.content,
                "source": d.source,
                "created_at": d.created_at
            } for d in docs]
        })
    
    return app


# ============================================================================
# Main
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="RAG Server Inteligente - Servidor HTTP Persistente",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Ejemplos:
  %(prog)s                          # Iniciar en puerto 11436
  %(prog)s --port 8080              # Puerto personalizado
  %(prog)s --embedding-model mpnet  # Modelo más preciso

Endpoints:
  curl http://localhost:11436/health
  curl -X POST http://localhost:11436/query \\
       -H "Content-Type: application/json" \\
       -d '{"question": "¿Cuál es la capital de Francia?"}'
        """
    )
    
    parser.add_argument(
        "--port", "-p", 
        type=int, 
        default=DEFAULT_RAG_PORT,
        help=f"Puerto del servidor (default: {DEFAULT_RAG_PORT})"
    )
    parser.add_argument(
        "--host", 
        default="0.0.0.0",
        help="Host a escuchar (default: 0.0.0.0)"
    )
    parser.add_argument(
        "--llm-url", 
        default=DEFAULT_LLM_URL,
        help=f"URL del LLM (default: {DEFAULT_LLM_URL})"
    )
    parser.add_argument(
        "--embedding-model", "-e",
        choices=list(EMBEDDING_MODELS.keys()),
        default="minilm",
        help="Modelo de embeddings (default: minilm)"
    )
    parser.add_argument(
        "--debug", 
        action="store_true",
        help="Modo debug"
    )
    
    args = parser.parse_args()
    
    print("""
╔═══════════════════════════════════════════════════════════════╗
║           🧠 RAG Server Inteligente - neuro-bitnet            ║
╠═══════════════════════════════════════════════════════════════╣
║  Servidor persistente con clasificación automática de queries ║
║                                                               ║
║  El modelo de embeddings se carga UNA vez y permanece         ║
║  en memoria para respuestas rápidas.                          ║
╚═══════════════════════════════════════════════════════════════╝
    """)
    
    print(f"📋 Configuración:")
    print(f"   Puerto: {args.port}")
    print(f"   LLM URL: {args.llm_url}")
    print(f"   Modelo embeddings: {args.embedding_model}")
    print()
    
    # Crear sistema RAG
    rag = SmartRAG(
        embedding_model=args.embedding_model,
        llm_url=args.llm_url
    )
    
    # Crear y ejecutar app
    app = create_app(rag)
    
    print(f"\n🚀 Servidor listo en http://{args.host}:{args.port}")
    print(f"\n📡 Endpoints:")
    print(f"   POST /query      - Consulta inteligente")
    print(f"   POST /classify   - Solo clasificar")
    print(f"   POST /add        - Agregar documento")
    print(f"   POST /search     - Buscar en RAG")
    print(f"   GET  /health     - Estado del servidor")
    print(f"   GET  /stats      - Estadísticas")
    print(f"   GET  /documents  - Listar documentos")
    print()
    
    app.run(host=args.host, port=args.port, debug=args.debug, threaded=True)


if __name__ == "__main__":
    main()
