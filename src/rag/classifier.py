"""
Query Classifier
================

Clasificador inteligente de consultas que determina la mejor estrategia de respuesta.

Basado en análisis de benchmarks:
- Matemáticas: 100% precisión sin RAG
- Razonamiento: 100% precisión sin RAG  
- Código genérico: 100% precisión sin RAG
- Tools: 100% precisión sin RAG
- Chat factual: 66.7% precisión → NECESITA RAG (+33% mejora)
"""

import re
from enum import Enum
from dataclasses import dataclass, field
from typing import List, Tuple, Pattern


class QueryCategory(Enum):
    """Categorías de consultas."""
    MATH = "math"                      # Operaciones matemáticas
    CODE = "code"                      # Programación y código
    REASONING = "reasoning"            # Razonamiento lógico
    TOOLS = "tools"                    # Llamadas a herramientas
    GREETING = "greeting"              # Saludos y despedidas
    FACTUAL = "factual"               # Conocimiento factual (requiere RAG/web)
    CONVERSATIONAL = "conversational"  # Conversación general


class QueryStrategy(Enum):
    """Estrategia de respuesta."""
    LLM_DIRECT = "llm_direct"          # Respuesta directa del LLM
    RAG_LOCAL = "rag_local"            # Usar RAG local únicamente
    RAG_THEN_WEB = "rag_then_web"      # RAG primero, web si no hay info
    WEB_SEARCH = "web_search"          # Búsqueda web directa


@dataclass
class ClassificationResult:
    """
    Resultado de clasificación de una consulta.
    
    Attributes:
        category: Categoría detectada
        strategy: Estrategia recomendada
        confidence: Nivel de confianza (0.0 a 1.0)
        reasons: Lista de razones para la clasificación
    """
    category: QueryCategory
    strategy: QueryStrategy
    confidence: float
    reasons: List[str] = field(default_factory=list)
    
    def to_dict(self) -> dict:
        """Convierte a diccionario serializable."""
        return {
            "category": self.category.value,
            "strategy": self.strategy.value,
            "confidence": self.confidence,
            "reasons": self.reasons
        }


class QueryClassifier:
    """
    Clasifica consultas para determinar la mejor estrategia de respuesta.
    
    Utiliza patrones regex para identificar el tipo de consulta y
    asignar la estrategia óptima basada en benchmarks de precisión.
    
    Example:
        classifier = QueryClassifier()
        result = classifier.classify("¿Cuál es la capital de Francia?")
        print(result.category)  # QueryCategory.FACTUAL
        print(result.strategy)  # QueryStrategy.RAG_THEN_WEB
    """
    
    # Patrones de matemáticas (preguntas directas de cálculo)
    MATH_PATTERNS = [
        r'^\d+\s*[\+\-\*\/\^]\s*\d+',           # 5+3, 10*2 (expresión pura)
        r'cuánto\s+es\s+\d+',                    # cuánto es 5+3
        r'resultado\s+de\s+\d+',                 # resultado de 5+3
        r'raíz\s*(cuadrada|cúbica)',            # raíces
        r'porcentaje|%',                         # porcentajes
    ]
    
    # Patrones de código
    CODE_PATTERNS = [
        r'escribe?\s+(un\s+)?(código|programa|función|script)',
        r'crea\s+(una?\s+)?(función|clase|método)',
        r'cómo\s+(se\s+)?(programa|codifica|implementa)',
        r'python|javascript|java|c\+\+|rust|go|typescript',
        r'función\s+que|método\s+que',
        r'bucle|loop|for|while|if\s+else',
        r'variable|array|lista|diccionario',
        r'print|console\.log|printf',
        r'def\s+\w+|function\s+\w+|class\s+\w+',
        r'hola\s+mundo|hello\s+world',
        r'ordenar|sort|filtrar|filter|mapear|map',
    ]
    
    # Patrones de razonamiento
    REASONING_PATTERNS = [
        r'si\s+.+\s+entonces',                   # silogismos
        r'qué\s+sigue|siguiente\s+número',       # secuencias
        r'secuencia|patrón|serie',
        r'lógica|lógicamente|deduce|deducir',
        r'todos\s+los\s+.+\s+son',               # silogismos clásicos
        r'por\s+lo\s+tanto|en\s+consecuencia',
        r'verdadero\s+o\s+falso',
        r'paradoja|acertijo|puzzle',
    ]
    
    # Patrones de tools/funciones (comandos para herramientas)
    TOOLS_PATTERNS = [
        r'clima\s+(en|de)|weather',
        r'traducir?\s+.+\s+(a|al)|traduce?\s+.+\s+(a|al)',
        r'traduce?\s+(esto|eso|el|la|lo)\s+(a|al)',
        r'translate',
        r'buscar?\s+(información|info)',
        r'enviar?\s+(un\s+)?(mensaje|email|correo)',
        r'mensaje\s+(a|para)\s+\w+',
        r'crear?\s+(un\s+)?(evento|recordatorio|alarma)',
        r'recordatorio\b',
        r'obtener?\s+(datos|información)',
        r'calcula(r|dora)?\s+\d+',               # calcula 25*4 (comando para herramienta)
        r'suma\s+\d+|resta\s+\d+|multiplica\s+\d+|divide\s+\d+',
    ]
    
    # Patrones de saludos
    GREETING_PATTERNS = [
        r'^hola\b|^hi\b|^hey\b',
        r'^buenas?\s*(días|tardes|noches)',
        r'^buenos\s+(días|tardes|noches)',
        r'^saludos',
        r'^qué\s+tal|^cómo\s+estás',
        r'^adiós|^hasta\s+(luego|pronto|mañana)',
        r'^chao|^bye',
        r'^gracias|^thank',
    ]
    
    # Patrones de conocimiento factual (requieren RAG)
    FACTUAL_PATTERNS = [
        r'(quién|quien)\s+(es|fue|era)',         # quién es Einstein
        r'(qué|que)\s+es\s+(un|una|el|la|este)', # qué es la fotosíntesis
        r'capital\s+de|capital\s+del',           # capital de Francia
        r'cuándo\s+(nació|murió|fue|ocurrió)',   # fechas históricas
        r'dónde\s+(está|queda|nació)',           # lugares
        r'historia\s+de|origen\s+de',
        r'inventor\s+de|creador\s+de|fundador',
        r'inventó|descubrió|creó|fundó',
        r'presidente\s+de|rey\s+de|líder',
        r'población\s+de|habitantes',
        r'(qué|cuál)\s+país|en\s+qué\s+año',
        r'descubrimiento|invención|revolución',
        r'siglo\s+\w+|año\s+\d+',
        r'guerra|batalla|tratado',
        r'científico|artista|escritor|músico',
        r'empresa|compañía|corporación',
        r'película|libro|canción|álbum',
        # Preguntas sobre proyecto/código indexado
        r'(qué|cómo)\s+(hace|funciona|es)\s+(la|el|este)',
        r'(qué|cuáles)\s+(clases|funciones|métodos)\s+hay',
        r'este\s+proyecto|el\s+proyecto',
        r'en\s+(el|este)\s+(archivo|código|script)',
        r'(ejecutar|correr|usar)\s+(con|el)\s+docker',
    ]
    
    def __init__(self):
        """Inicializa el clasificador compilando los patrones regex."""
        self._math_re = self._compile_patterns(self.MATH_PATTERNS)
        self._code_re = self._compile_patterns(self.CODE_PATTERNS)
        self._reasoning_re = self._compile_patterns(self.REASONING_PATTERNS)
        self._tools_re = self._compile_patterns(self.TOOLS_PATTERNS)
        self._greeting_re = self._compile_patterns(self.GREETING_PATTERNS)
        self._factual_re = self._compile_patterns(self.FACTUAL_PATTERNS)
    
    @staticmethod
    def _compile_patterns(patterns: List[str]) -> List[Pattern]:
        """Compila lista de patrones regex."""
        return [re.compile(p, re.IGNORECASE) for p in patterns]
    
    def _count_matches(self, text: str, patterns: List[Pattern]) -> Tuple[int, List[str]]:
        """Cuenta cuántos patrones coinciden y retorna los que sí."""
        matches = []
        for pattern in patterns:
            if pattern.search(text):
                matches.append(pattern.pattern)
        return len(matches), matches
    
    def classify(self, query: str) -> ClassificationResult:
        """
        Clasifica una consulta y determina la mejor estrategia.
        
        Args:
            query: Texto de la consulta del usuario
            
        Returns:
            ClassificationResult con categoría, estrategia y confianza
        """
        query_clean = query.strip().lower()
        reasons = []
        
        # Conteo de coincidencias por categoría con pesos
        scores = {}
        
        math_count, math_matches = self._count_matches(query_clean, self._math_re)
        scores[QueryCategory.MATH] = math_count * 2.0  # Peso alto para matemáticas
        
        code_count, code_matches = self._count_matches(query_clean, self._code_re)
        scores[QueryCategory.CODE] = code_count * 1.5
        
        reasoning_count, reasoning_matches = self._count_matches(query_clean, self._reasoning_re)
        scores[QueryCategory.REASONING] = reasoning_count * 1.5
        
        tools_count, tools_matches = self._count_matches(query_clean, self._tools_re)
        scores[QueryCategory.TOOLS] = tools_count * 2.0
        
        greeting_count, greeting_matches = self._count_matches(query_clean, self._greeting_re)
        scores[QueryCategory.GREETING] = greeting_count * 2.5  # Saludos son muy específicos
        
        factual_count, factual_matches = self._count_matches(query_clean, self._factual_re)
        scores[QueryCategory.FACTUAL] = factual_count * 1.8
        
        # Encontrar categoría ganadora
        max_score = max(scores.values()) if scores else 0
        
        if max_score == 0:
            # Sin coincidencias claras -> conversacional
            category = QueryCategory.CONVERSATIONAL
            strategy = QueryStrategy.LLM_DIRECT
            confidence = 0.5
            reasons.append("Sin patrones claros detectados")
        else:
            # Categoría con mayor puntuación
            category = max(scores.keys(), key=lambda k: scores[k])
            total_score = sum(scores.values())
            confidence = min(0.95, max_score / max(total_score, 1) + 0.3)
            
            # Determinar estrategia según categoría
            strategy, reason = self._get_strategy_for_category(
                category, factual_matches, code_matches
            )
            reasons.append(reason)
        
        return ClassificationResult(
            category=category,
            strategy=strategy,
            confidence=confidence,
            reasons=reasons
        )
    
    def _get_strategy_for_category(
        self, 
        category: QueryCategory,
        factual_matches: List[str],
        code_matches: List[str]
    ) -> Tuple[QueryStrategy, str]:
        """
        Determina la estrategia óptima para una categoría.
        
        Solo FACTUAL usa RAG por defecto (mejora comprobada de +33%).
        CODE puede usar RAG local si hay documentación de proyecto.
        """
        if category == QueryCategory.FACTUAL:
            return (
                QueryStrategy.RAG_THEN_WEB,
                f"Factual → RAG + Web: {factual_matches[:2]}"
            )
        elif category == QueryCategory.CODE:
            return (
                QueryStrategy.RAG_LOCAL,
                f"Código → RAG local (docs proyecto): {code_matches[:2]}"
            )
        else:
            # MATH, REASONING, TOOLS, GREETING, CONVERSATIONAL
            # Van directo al LLM (mejor o igual precisión sin RAG)
            return (
                QueryStrategy.LLM_DIRECT,
                f"{category.value} → LLM directo (no necesita RAG)"
            )
