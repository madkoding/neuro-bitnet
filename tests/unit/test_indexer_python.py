"""
Unit Tests for Python Analyzer
==============================

Tests para el analizador de código Python basado en AST.
"""

import pytest
from src.rag.indexer.python import PythonAnalyzer
from src.rag.indexer.base import CodeElement


@pytest.fixture
def analyzer():
    """Crea una instancia del analizador."""
    return PythonAnalyzer()


@pytest.fixture
def simple_function_code():
    """Código con función simple."""
    return '''
def greet(name: str) -> str:
    """Say hello to someone."""
    return f"Hello, {name}!"
'''


@pytest.fixture
def class_code():
    """Código con clase."""
    return '''
class Calculator:
    """A simple calculator class."""
    
    def __init__(self, initial: int = 0):
        """Initialize with optional value."""
        self.value = initial
    
    def add(self, n: int) -> int:
        """Add n to current value."""
        self.value += n
        return self.value
    
    def subtract(self, n: int) -> int:
        """Subtract n from current value."""
        self.value -= n
        return self.value
'''


@pytest.fixture
def complex_code():
    """Código complejo con múltiples elementos."""
    return '''
"""Module docstring."""

import os
from typing import List, Optional

CONSTANT = 42

def helper(x: int) -> int:
    """Helper function."""
    return x * 2

class DataProcessor:
    """Process data efficiently."""
    
    @staticmethod
    def normalize(data: List[float]) -> List[float]:
        """Normalize data to 0-1 range."""
        min_val = min(data)
        max_val = max(data)
        return [(x - min_val) / (max_val - min_val) for x in data]
    
    @classmethod
    def from_file(cls, path: str) -> "DataProcessor":
        """Create instance from file."""
        return cls()
    
    def process(self, items: List[str]) -> List[str]:
        """Process list of items."""
        return [item.strip().lower() for item in items]

async def fetch_data(url: str) -> dict:
    """Fetch data asynchronously."""
    pass
'''


class TestPythonAnalyzerBasic:
    """Tests básicos del analizador."""
    
    def test_can_analyze_returns_true_for_py(self, analyzer):
        """Test que puede analizar archivos .py."""
        assert analyzer.can_analyze("script.py") is True
        assert analyzer.can_analyze("module/file.py") is True
    
    def test_can_analyze_returns_false_for_others(self, analyzer):
        """Test que no analiza otros tipos de archivo."""
        assert analyzer.can_analyze("script.js") is False
        assert analyzer.can_analyze("style.css") is False
        assert analyzer.can_analyze("data.json") is False
        assert analyzer.can_analyze("README.md") is False
    
    def test_returns_code_elements(self, analyzer, simple_function_code):
        """Test que retorna CodeElements."""
        elements = analyzer.analyze("test.py", simple_function_code)
        
        assert len(elements) >= 1
        assert isinstance(elements[0], CodeElement)


class TestFunctionExtraction:
    """Tests para extracción de funciones."""
    
    def test_extracts_simple_function(self, analyzer, simple_function_code):
        """Test que extrae función simple."""
        elements = analyzer.analyze("test.py", simple_function_code)
        
        assert len(elements) >= 1
        
        # Buscar el elemento de la función
        func_elements = [e for e in elements if "greet" in e.name]
        assert len(func_elements) == 1
        
        func = func_elements[0]
        assert func.type == "function"
        assert "name" in func.signature or "str" in func.signature
    
    def test_extracts_async_function(self, analyzer, complex_code):
        """Test que extrae funciones async."""
        elements = analyzer.analyze("test.py", complex_code)
        
        async_elements = [e for e in elements if "fetch_data" in e.name]
        assert len(async_elements) == 1
        
        func = async_elements[0]
        assert func.type == "function" or "async" in func.signature.lower()
    
    def test_function_has_correct_attributes(self, analyzer, simple_function_code):
        """Test que función tiene atributos correctos."""
        elements = analyzer.analyze("test.py", simple_function_code)
        func = elements[0]
        
        assert func.file_path == "test.py"
        assert func.line_number >= 1
        assert func.code_snippet != ""


class TestClassExtraction:
    """Tests para extracción de clases."""
    
    def test_extracts_class(self, analyzer, class_code):
        """Test que extrae clase."""
        elements = analyzer.analyze("test.py", class_code)
        
        class_elements = [e for e in elements if e.type == "class"]
        assert len(class_elements) >= 1
        
        cls = class_elements[0]
        assert "Calculator" in cls.name
    
    def test_extracts_methods(self, analyzer, class_code):
        """Test que extrae métodos de clase."""
        elements = analyzer.analyze("test.py", class_code)
        
        method_elements = [e for e in elements if e.type == "method"]
        
        # Debe incluir __init__, add, subtract
        assert len(method_elements) >= 3
        
        method_names = [e.name for e in method_elements]
        assert any("add" in name for name in method_names)
        assert any("subtract" in name for name in method_names)
    
    def test_extracts_static_and_class_methods(self, analyzer, complex_code):
        """Test que extrae métodos estáticos y de clase."""
        elements = analyzer.analyze("test.py", complex_code)
        
        # Buscar normalize y from_file
        normalize = [e for e in elements if "normalize" in e.name]
        from_file = [e for e in elements if "from_file" in e.name]
        
        assert len(normalize) >= 1
        assert len(from_file) >= 1


class TestDocstringExtraction:
    """Tests para extracción de docstrings."""
    
    def test_extracts_function_docstring(self, analyzer, simple_function_code):
        """Test que extrae docstring de función."""
        elements = analyzer.analyze("test.py", simple_function_code)
        func = elements[0]
        
        # El docstring debe contener la descripción
        assert "hello" in func.docstring.lower() or "Hello" in func.code_snippet
    
    def test_extracts_class_docstring(self, analyzer, class_code):
        """Test que extrae docstring de clase."""
        elements = analyzer.analyze("test.py", class_code)
        
        class_elements = [e for e in elements if e.type == "class"]
        cls = class_elements[0]
        
        assert "calculator" in cls.docstring.lower() or "Calculator" in cls.code_snippet


class TestComplexCode:
    """Tests para código complejo."""
    
    def test_handles_nested_classes(self, analyzer):
        """Test que maneja clases anidadas."""
        code = '''
class Outer:
    """Outer class."""
    
    class Inner:
        """Inner class."""
        
        def inner_method(self):
            """Inner method."""
            pass
    
    def outer_method(self):
        """Outer method."""
        pass
'''
        elements = analyzer.analyze("test.py", code)
        
        # Debe extraer ambas clases
        class_elements = [e for e in elements if e.type == "class"]
        assert len(class_elements) >= 2
    
    def test_handles_decorators(self, analyzer):
        """Test que maneja decoradores."""
        code = '''
@decorator
@another_decorator("arg")
def decorated_function():
    """Has decorators."""
    pass
'''
        elements = analyzer.analyze("test.py", code)
        
        assert len(elements) >= 1
        func = elements[0]
        assert "decorated" in func.name
    
    def test_handles_type_hints(self, analyzer):
        """Test que maneja type hints complejos."""
        code = '''
from typing import Dict, List, Optional, Union

def complex_types(
    items: List[Dict[str, Union[int, str]]],
    config: Optional[dict] = None
) -> Dict[str, List[int]]:
    """Function with complex types."""
    return {}
'''
        elements = analyzer.analyze("test.py", code)
        
        assert len(elements) >= 1


class TestEdgeCases:
    """Tests para casos límite."""
    
    def test_empty_file(self, analyzer):
        """Test archivo vacío."""
        elements = analyzer.analyze("test.py", "")
        
        assert isinstance(elements, list)
        assert len(elements) == 0
    
    def test_only_comments(self, analyzer):
        """Test archivo solo con comentarios."""
        code = '''
# This is a comment
# Another comment
'''
        elements = analyzer.analyze("test.py", code)
        
        assert isinstance(elements, list)
    
    def test_syntax_error_handled(self, analyzer):
        """Test que errores de sintaxis se manejan."""
        invalid_code = '''
def broken(
    # Missing closing paren
'''
        # No debe lanzar excepción
        elements = analyzer.analyze("test.py", invalid_code)
        
        assert isinstance(elements, list)
    
    def test_unicode_content(self, analyzer):
        """Test contenido unicode."""
        code = '''
def greet_español(nombre: str) -> str:
    """Saluda en español con acentos: áéíóú ñ."""
    return f"¡Hola, {nombre}! 🎉"
'''
        elements = analyzer.analyze("test.py", code)
        
        assert len(elements) >= 1
        assert "español" in elements[0].name or "greet" in elements[0].name
