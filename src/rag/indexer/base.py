"""
Abstract Code Analyzer
======================

Define la interfaz común para analizadores de código.
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import List


@dataclass
class CodeElement:
    """
    Representa un elemento de código extraído (función, clase, etc.).
    
    Attributes:
        name: Nombre del elemento
        type: Tipo (function, class, method, variable, etc.)
        docstring: Documentación del elemento
        signature: Firma/declaración del elemento
        file_path: Ruta del archivo fuente
        line_number: Número de línea donde se define
        code_snippet: Fragmento de código (limitado en tamaño)
    """
    name: str
    type: str
    docstring: str
    signature: str
    file_path: str
    line_number: int
    code_snippet: str
    
    def to_markdown(self) -> str:
        """Convierte el elemento a formato Markdown."""
        md = f"### {self.name}\n\n"
        md += f"**Tipo:** {self.type}\n\n"
        md += f"```\n{self.signature}\n```\n\n"
        if self.docstring:
            md += f"{self.docstring}\n\n"
        md += f"*Definido en {self.file_path}:{self.line_number}*\n"
        return md


class ABCAnalyzer(ABC):
    """
    Interfaz abstracta para analizadores de código.
    
    Cada analizador debe implementar el método analyze() que extrae
    elementos de código de un archivo fuente.
    
    Example:
        class MyAnalyzer(ABCAnalyzer):
            def analyze(self, file_path: str, content: str) -> List[CodeElement]:
                # Implementación específica
                pass
    """
    
    @abstractmethod
    def analyze(self, file_path: str, content: str) -> List[CodeElement]:
        """
        Analiza el contenido de un archivo y extrae elementos de código.
        
        Args:
            file_path: Ruta del archivo (para referencia)
            content: Contenido del archivo
            
        Returns:
            Lista de CodeElement encontrados
        """
        pass
    
    @property
    @abstractmethod
    def supported_extensions(self) -> List[str]:
        """
        Extensiones de archivo soportadas por este analizador.
        
        Returns:
            Lista de extensiones (ej: ['.py', '.pyw'])
        """
        pass
    
    def can_analyze(self, file_path: str) -> bool:
        """
        Determina si este analizador puede procesar un archivo.
        
        Args:
            file_path: Ruta o nombre del archivo
            
        Returns:
            True si el analizador soporta la extensión del archivo
        """
        import os
        ext = os.path.splitext(file_path)[1].lower()
        return ext in self.supported_extensions
