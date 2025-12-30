#!/usr/bin/env python3
"""
RAG Client - Cliente HTTP para el servidor RAG
==============================================

Cliente de línea de comandos para interactuar con el servidor RAG.

Uso:
    python -m src.cli.rag_client "¿Cuál es la capital de Francia?"
    python -m src.cli.rag_client --classify "25+17"
    python -m src.cli.rag_client --add "Python es un lenguaje de programación"
    python -m src.cli.rag_client --search "Python"
    python -m src.cli.rag_client --stats
"""

import argparse
import json
import sys

import requests

from src.rag.core import DEFAULT_RAG_PORT


def print_json(data: dict, indent: int = 2) -> None:
    """Imprime JSON formateado."""
    print(json.dumps(data, indent=indent, ensure_ascii=False))


def query(url: str, question: str, strategy: str | None = None) -> dict:
    """Envía consulta al servidor."""
    payload = {"question": question}
    if strategy:
        payload["strategy"] = strategy
    
    response = requests.post(f"{url}/query", json=payload, timeout=120)
    response.raise_for_status()
    return response.json()


def classify(url: str, question: str) -> dict:
    """Clasifica una consulta sin ejecutarla."""
    response = requests.post(
        f"{url}/classify", 
        json={"question": question}, 
        timeout=10
    )
    response.raise_for_status()
    return response.json()


def add_document(url: str, content: str, source: str = "manual") -> dict:
    """Agrega un documento al RAG."""
    response = requests.post(
        f"{url}/add",
        json={"content": content, "source": source},
        timeout=30
    )
    response.raise_for_status()
    return response.json()


def search(url: str, query_text: str, top_k: int = 5) -> dict:
    """Busca documentos similares."""
    response = requests.post(
        f"{url}/search",
        json={"query": query_text, "top_k": top_k},
        timeout=30
    )
    response.raise_for_status()
    return response.json()


def get_stats(url: str) -> dict:
    """Obtiene estadísticas del servidor."""
    response = requests.get(f"{url}/stats", timeout=10)
    response.raise_for_status()
    return response.json()


def get_health(url: str) -> dict:
    """Verifica estado del servidor."""
    response = requests.get(f"{url}/health", timeout=10)
    response.raise_for_status()
    return response.json()


def main():
    parser = argparse.ArgumentParser(
        description="Cliente para el servidor RAG inteligente",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Ejemplos:
  %(prog)s "¿Cuál es la capital de Francia?"    # Consulta normal
  %(prog)s --classify "25+17"                   # Solo clasificar
  %(prog)s --add "Python es genial"             # Agregar documento
  %(prog)s --search "Python"                    # Buscar documentos
  %(prog)s --stats                              # Ver estadísticas
        """
    )
    
    parser.add_argument(
        "question",
        nargs="?",
        help="Pregunta a realizar"
    )
    parser.add_argument(
        "--url",
        default=f"http://localhost:{DEFAULT_RAG_PORT}",
        help=f"URL del servidor RAG (default: http://localhost:{DEFAULT_RAG_PORT})"
    )
    parser.add_argument(
        "--classify", "-c",
        action="store_true",
        help="Solo clasificar la consulta sin ejecutarla"
    )
    parser.add_argument(
        "--add", "-a",
        metavar="CONTENT",
        help="Agregar documento al RAG"
    )
    parser.add_argument(
        "--source",
        default="manual",
        help="Fuente del documento (para --add)"
    )
    parser.add_argument(
        "--search", "-s",
        metavar="QUERY",
        help="Buscar documentos similares"
    )
    parser.add_argument(
        "--top-k",
        type=int,
        default=5,
        help="Número de resultados (para --search)"
    )
    parser.add_argument(
        "--stats",
        action="store_true",
        help="Mostrar estadísticas del servidor"
    )
    parser.add_argument(
        "--health",
        action="store_true",
        help="Verificar estado del servidor"
    )
    parser.add_argument(
        "--strategy",
        choices=["llm_direct", "rag_local", "rag_then_web", "web_search"],
        help="Forzar estrategia específica"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Salida en formato JSON"
    )
    
    args = parser.parse_args()
    
    try:
        # Verificar conexión primero
        if args.health:
            result = get_health(args.url)
            if args.json:
                print_json(result)
            else:
                status = "✅ OK" if result.get("status") == "healthy" else "❌ Error"
                print(f"Estado: {status}")
                print(f"Embeddings cargados: {result.get('embedding_model_loaded', False)}")
                print(f"LLM disponible: {result.get('llm_available', False)}")
            return 0
        
        if args.stats:
            result = get_stats(args.url)
            if args.json:
                print_json(result)
            else:
                print("📊 Estadísticas del servidor:")
                print(f"   Total queries: {result.get('queries_total', 0)}")
                print(f"   Por categoría: {result.get('queries_by_category', {})}")
                print(f"   Búsquedas web: {result.get('web_searches', 0)}")
                print(f"   Modelo: {result.get('embedding_model', 'unknown')}")
            return 0
        
        if args.add:
            result = add_document(args.url, args.add, args.source)
            if args.json:
                print_json(result)
            else:
                print(f"✅ Documento agregado con ID: {result.get('id')}")
            return 0
        
        if args.search:
            result = search(args.url, args.search, args.top_k)
            if args.json:
                print_json(result)
            else:
                results = result.get("results", [])
                if not results:
                    print("No se encontraron documentos similares")
                else:
                    print(f"🔍 {len(results)} resultados encontrados:\n")
                    for i, r in enumerate(results, 1):
                        print(f"[{i}] Score: {r['score']:.2f}")
                        print(f"    Fuente: {r['source']}")
                        content = r['content'][:200] + "..." if len(r['content']) > 200 else r['content']
                        print(f"    {content}\n")
            return 0
        
        if not args.question:
            parser.print_help()
            return 1
        
        if args.classify:
            result = classify(args.url, args.question)
            if args.json:
                print_json(result)
            else:
                print(f"📋 Clasificación:")
                print(f"   Categoría: {result.get('category')}")
                print(f"   Estrategia: {result.get('strategy')}")
                print(f"   Confianza: {result.get('confidence', 0):.0%}")
                print(f"   Razones: {result.get('reasons', [])}")
            return 0
        
        # Consulta normal
        result = query(args.url, args.question, args.strategy)
        
        if args.json:
            print_json(result)
        else:
            classification = result.get("classification", {})
            print(f"🏷️  [{classification.get('category')} / {classification.get('strategy')}]")
            print()
            print(result.get("answer", "Sin respuesta"))
            print()
            
            sources = result.get("sources", [])
            if sources:
                print(f"📚 Fuentes ({len(sources)}):")
                for s in sources[:3]:
                    print(f"   - {s.get('source', 'unknown')}: score {s.get('score', 0):.2f}")
            
            timing = result.get("timing", {})
            print(f"\n⏱️  {timing.get('total_ms', 0)}ms")
        
        return 0
        
    except requests.exceptions.ConnectionError:
        print(f"❌ No se puede conectar al servidor en {args.url}")
        print("   Asegúrate de que el servidor está corriendo:")
        print("   python -m src.server.rag_server")
        return 1
    except requests.exceptions.HTTPError as e:
        print(f"❌ Error HTTP: {e}")
        return 1
    except Exception as e:
        print(f"❌ Error: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
