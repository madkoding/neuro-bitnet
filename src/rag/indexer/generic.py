"""
Generic Code Analyzer
=====================

Analiza código usando regex para lenguajes sin AST disponible.
"""

import re
from typing import List, Tuple, Dict

from src.rag.indexer.base import ABCAnalyzer, CodeElement


class GenericAnalyzer(ABCAnalyzer):
    """
    Analizador genérico de código usando expresiones regulares.
    
    Soporta múltiples lenguajes con patrones predefinidos.
    Menos preciso que el análisis AST pero funciona para cualquier lenguaje.
    
    Lenguajes soportados:
    - JavaScript/TypeScript
    - Go
    - Rust
    - Java
    - C/C++
    - Ruby
    - PHP
    
    Example:
        analyzer = GenericAnalyzer()
        elements = analyzer.analyze("main.js", source_code, "javascript")
    """
    
    # Mapeo de extensión a lenguaje
    EXTENSION_MAP: Dict[str, str] = {
        '.js': 'javascript',
        '.jsx': 'javascript',
        '.ts': 'typescript',
        '.tsx': 'typescript',
        '.go': 'go',
        '.rs': 'rust',
        '.java': 'java',
        '.c': 'c',
        '.h': 'c',
        '.cpp': 'cpp',
        '.hpp': 'cpp',
        '.cc': 'cpp',
        '.rb': 'ruby',
        '.php': 'php',
    }
    
    # Patrones por lenguaje: (regex, tipo_elemento)
    PATTERNS: Dict[str, List[Tuple[str, str]]] = {
        'javascript': [
            (r'(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\([^)]*\)', 'function'),
            (r'(?:export\s+)?class\s+(\w+)', 'class'),
            (r'const\s+(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>', 'function'),
            (r'(?:export\s+)?const\s+(\w+)\s*=\s*\{', 'object'),
        ],
        'typescript': [
            (r'(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*[<(]', 'function'),
            (r'(?:export\s+)?class\s+(\w+)', 'class'),
            (r'(?:export\s+)?interface\s+(\w+)', 'interface'),
            (r'(?:export\s+)?type\s+(\w+)\s*=', 'type'),
            (r'(?:export\s+)?enum\s+(\w+)', 'enum'),
        ],
        'go': [
            (r'func\s+(?:\([^)]+\)\s+)?(\w+)\s*\(', 'function'),
            (r'type\s+(\w+)\s+struct', 'struct'),
            (r'type\s+(\w+)\s+interface', 'interface'),
        ],
        'rust': [
            (r'(?:pub\s+)?(?:async\s+)?fn\s+(\w+)', 'function'),
            (r'(?:pub\s+)?struct\s+(\w+)', 'struct'),
            (r'(?:pub\s+)?trait\s+(\w+)', 'trait'),
            (r'(?:pub\s+)?enum\s+(\w+)', 'enum'),
            (r'impl(?:<[^>]+>)?\s+(\w+)', 'impl'),
        ],
        'java': [
            (r'(?:public|private|protected)?\s*(?:static\s+)?(?:final\s+)?\w+\s+(\w+)\s*\(', 'method'),
            (r'(?:public|private|protected)?\s*class\s+(\w+)', 'class'),
            (r'(?:public|private|protected)?\s*interface\s+(\w+)', 'interface'),
            (r'(?:public|private|protected)?\s*enum\s+(\w+)', 'enum'),
        ],
        'c': [
            (r'(?:\w+\s+)+(\w+)\s*\([^)]*\)\s*\{', 'function'),
            (r'typedef\s+struct\s+\w*\s*\{[^}]*\}\s*(\w+)', 'struct'),
            (r'struct\s+(\w+)\s*\{', 'struct'),
        ],
        'cpp': [
            (r'(?:\w+\s+)+(\w+)\s*\([^)]*\)\s*(?:const\s*)?\{', 'function'),
            (r'class\s+(\w+)', 'class'),
            (r'struct\s+(\w+)', 'struct'),
            (r'namespace\s+(\w+)', 'namespace'),
        ],
        'ruby': [
            (r'def\s+(\w+)', 'method'),
            (r'class\s+(\w+)', 'class'),
            (r'module\s+(\w+)', 'module'),
        ],
        'php': [
            (r'function\s+(\w+)\s*\(', 'function'),
            (r'class\s+(\w+)', 'class'),
            (r'interface\s+(\w+)', 'interface'),
            (r'trait\s+(\w+)', 'trait'),
        ],
    }
    
    @property
    def supported_extensions(self) -> List[str]:
        return list(self.EXTENSION_MAP.keys())
    
    def analyze(
        self, 
        file_path: str, 
        content: str, 
        language: str | None = None
    ) -> List[CodeElement]:
        """
        Analiza código extrayendo elementos usando regex.
        
        Args:
            file_path: Ruta del archivo
            content: Código fuente
            language: Lenguaje (opcional, se detecta por extensión)
            
        Returns:
            Lista de CodeElement encontrados
        """
        # Detectar lenguaje por extensión si no se especifica
        if language is None:
            import os
            ext = os.path.splitext(file_path)[1].lower()
            language = self.EXTENSION_MAP.get(ext, 'unknown')
        
        patterns = self.PATTERNS.get(language, [])
        elements = []
        lines = content.split('\n')
        
        for pattern, elem_type in patterns:
            try:
                for match in re.finditer(pattern, content, re.MULTILINE):
                    name = match.group(1)
                    
                    # Calcular número de línea
                    line_number = content[:match.start()].count('\n') + 1
                    
                    # Extraer snippet de contexto (10 líneas)
                    start_line = max(0, line_number - 1)
                    end_line = min(len(lines), line_number + 9)
                    snippet = '\n'.join(lines[start_line:end_line])
                    
                    elements.append(CodeElement(
                        name=name,
                        type=elem_type,
                        docstring="",  # Regex no extrae docstrings fácilmente
                        signature=match.group(0).strip(),
                        file_path=file_path,
                        line_number=line_number,
                        code_snippet=snippet[:500]
                    ))
            except re.error:
                # Patrón inválido, continuar con el siguiente
                continue
        
        return elements
    
    def get_language(self, file_path: str) -> str:
        """
        Detecta el lenguaje basándose en la extensión del archivo.
        
        Args:
            file_path: Ruta del archivo
            
        Returns:
            Nombre del lenguaje o 'unknown'
        """
        import os
        ext = os.path.splitext(file_path)[1].lower()
        return self.EXTENSION_MAP.get(ext, 'unknown')
