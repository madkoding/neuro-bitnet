"""
Python Code Analyzer
====================

Analiza código Python usando AST para extracción precisa.
"""

import ast
from typing import List

from src.rag.indexer.base import ABCAnalyzer, CodeElement


class PythonAnalyzer(ABCAnalyzer):
    """
    Analizador de código Python usando Abstract Syntax Tree (AST).
    
    Extrae:
    - Funciones (sync y async)
    - Clases y sus métodos
    - Docstrings
    - Signatures con type hints
    
    Example:
        analyzer = PythonAnalyzer()
        elements = analyzer.analyze("main.py", source_code)
        for elem in elements:
            print(f"{elem.type}: {elem.name}")
    """
    
    @property
    def supported_extensions(self) -> List[str]:
        return ['.py', '.pyw']
    
    def analyze(self, file_path: str, content: str) -> List[CodeElement]:
        """
        Analiza código Python extrayendo funciones y clases.
        
        Args:
            file_path: Ruta del archivo
            content: Código fuente Python
            
        Returns:
            Lista de CodeElement (funciones, clases, métodos)
        """
        elements = []
        
        try:
            tree = ast.parse(content)
            
            for node in ast.walk(tree):
                if isinstance(node, ast.FunctionDef):
                    elements.append(self._extract_function(node, file_path))
                elif isinstance(node, ast.AsyncFunctionDef):
                    elements.append(self._extract_function(node, file_path, is_async=True))
                elif isinstance(node, ast.ClassDef):
                    elements.append(self._extract_class(node, file_path))
                    # Extraer métodos de la clase
                    for item in node.body:
                        if isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef)):
                            is_async = isinstance(item, ast.AsyncFunctionDef)
                            elem = self._extract_function(item, file_path, is_async)
                            elem.name = f"{node.name}.{elem.name}"
                            elem.type = "method"
                            elements.append(elem)
        except SyntaxError:
            # Archivo con errores de sintaxis, ignorar
            pass
        
        return elements
    
    def _extract_function(
        self, 
        node: ast.FunctionDef | ast.AsyncFunctionDef, 
        file_path: str, 
        is_async: bool = False
    ) -> CodeElement:
        """Extrae información de una función."""
        # Construir argumentos con type hints
        args = []
        for arg in node.args.args:
            arg_str = arg.arg
            if arg.annotation:
                try:
                    arg_str += f": {ast.unparse(arg.annotation)}"
                except Exception:
                    pass
            args.append(arg_str)
        
        # Construir signature
        async_prefix = "async " if is_async else ""
        signature = f"{async_prefix}def {node.name}({', '.join(args)})"
        
        # Añadir return type si existe
        if node.returns:
            try:
                signature += f" -> {ast.unparse(node.returns)}"
            except Exception:
                pass
        
        # Obtener snippet de código
        try:
            code_snippet = ast.unparse(node)[:500]
        except Exception:
            code_snippet = ""
        
        return CodeElement(
            name=node.name,
            type="function",
            docstring=ast.get_docstring(node) or "",
            signature=signature,
            file_path=file_path,
            line_number=node.lineno,
            code_snippet=code_snippet
        )
    
    def _extract_class(self, node: ast.ClassDef, file_path: str) -> CodeElement:
        """Extrae información de una clase."""
        # Obtener bases
        bases = []
        for base in node.bases:
            try:
                bases.append(ast.unparse(base))
            except Exception:
                pass
        
        # Construir signature
        signature = f"class {node.name}"
        if bases:
            signature += f"({', '.join(bases)})"
        
        # Obtener atributos de clase y métodos para el snippet
        methods = []
        for item in node.body:
            if isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef)):
                methods.append(item.name)
        
        snippet = f"{signature}:\n"
        if methods:
            snippet += "    # Methods: " + ", ".join(methods[:10])
        
        return CodeElement(
            name=node.name,
            type="class",
            docstring=ast.get_docstring(node) or "",
            signature=signature,
            file_path=file_path,
            line_number=node.lineno,
            code_snippet=snippet
        )
