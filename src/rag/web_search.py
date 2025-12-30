"""
Web Search Module
=================

Proporciona búsqueda de información en fuentes web (Wikipedia).
"""

import logging
from typing import List, Dict, Optional

logger = logging.getLogger(__name__)


class WebSearcher:
    """
    Busca información en Wikipedia para enriquecer el contexto RAG.
    
    Example:
        searcher = WebSearcher()
        result = searcher.search_wikipedia("Albert Einstein")
        if result:
            print(result['content'])
    """
    
    def __init__(self):
        self.headers = {
            "User-Agent": "Mozilla/5.0 (compatible; neuro-bitnet RAG/1.0)"
        }
    
    def search_wikipedia(self, query: str, lang: str = "es") -> Optional[Dict]:
        """
        Busca en Wikipedia y retorna el resumen del artículo.
        
        Args:
            query: Término de búsqueda
            lang: Código de idioma (es, en, etc.)
            
        Returns:
            Diccionario con title, content, source, type o None si no hay resultado
        """
        try:
            import requests
            from urllib.parse import quote_plus
            
            # Intento 1: Búsqueda directa por título
            url = f"https://{lang}.wikipedia.org/api/rest_v1/page/summary/{quote_plus(query)}"
            response = requests.get(url, headers=self.headers, timeout=10)
            
            if response.status_code == 200:
                data = response.json()
                if data.get("extract") and data.get("type") != "disambiguation":
                    return {
                        "title": data.get("title", query),
                        "content": data["extract"],
                        "source": data.get("content_urls", {}).get("desktop", {}).get("page", "Wikipedia"),
                        "type": "wikipedia"
                    }
            
            # Intento 2: Búsqueda API
            search_url = f"https://{lang}.wikipedia.org/w/api.php"
            params = {
                "action": "query",
                "list": "search",
                "srsearch": query,
                "format": "json",
                "srlimit": 1
            }
            response = requests.get(search_url, params=params, headers=self.headers, timeout=10)
            
            if response.status_code == 200:
                data = response.json()
                search_results = data.get("query", {}).get("search", [])
                if search_results:
                    title = search_results[0]["title"]
                    # Obtener resumen del artículo encontrado
                    url = f"https://{lang}.wikipedia.org/api/rest_v1/page/summary/{quote_plus(title)}"
                    response = requests.get(url, headers=self.headers, timeout=10)
                    if response.status_code == 200:
                        data = response.json()
                        if data.get("extract"):
                            return {
                                "title": data.get("title", title),
                                "content": data["extract"],
                                "source": "Wikipedia",
                                "type": "wikipedia"
                            }
            
            return None
            
        except Exception as e:
            logger.warning(f"Error en Wikipedia: {e}")
            return None
    
    def search(self, query: str) -> List[Dict]:
        """
        Búsqueda combinada extrayendo términos clave.
        
        Args:
            query: Consulta del usuario
            
        Returns:
            Lista de resultados encontrados
        """
        results = []
        
        # Extraer términos clave de la consulta
        terms = self._extract_terms(query)
        
        for term in terms[:2]:  # Máximo 2 búsquedas
            result = self.search_wikipedia(term)
            if result:
                results.append(result)
                break  # Un resultado es suficiente
        
        return results
    
    def _extract_terms(self, query: str) -> List[str]:
        """
        Extrae términos de búsqueda de una consulta.
        
        Args:
            query: Consulta completa del usuario
            
        Returns:
            Lista de términos de búsqueda ordenados por relevancia
        """
        stop_words = {
            'qué', 'que', 'quién', 'quien', 'cuál', 'cual', 'cómo', 'como',
            'dónde', 'donde', 'cuándo', 'cuando', 'el', 'la', 'los', 'las',
            'un', 'una', 'de', 'del', 'al', 'a', 'en', 'con', 'es', 'son',
            'capital', 'fue', 'era', 'y', 'o', 'se', 'su'
        }
        
        # Limpiar query
        clean = query.lower()
        clean = ''.join(c if c.isalnum() or c.isspace() else ' ' for c in clean)
        words = [w for w in clean.split() if w not in stop_words and len(w) > 2]
        
        # Retornar: términos combinados primero, luego individuales
        return [' '.join(words)] + [w.capitalize() for w in words if len(w) > 3]
