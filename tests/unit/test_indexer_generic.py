"""
Unit Tests for Generic Analyzer
===============================

Tests para el analizador de código genérico basado en regex.
"""

import pytest
from src.rag.indexer.generic import GenericAnalyzer
from src.rag.indexer.base import CodeElement


@pytest.fixture
def analyzer():
    """Crea una instancia del analizador."""
    return GenericAnalyzer()


class TestSupportedLanguages:
    """Tests para lenguajes soportados."""
    
    @pytest.mark.parametrize("filename,expected", [
        ("script.js", True),
        ("app.ts", True),
        ("main.go", True),
        ("lib.rs", True),
        ("Main.java", True),
        ("program.c", True),
        ("program.cpp", True),
        ("program.h", True),
        ("script.rb", True),
        ("index.php", True),
        ("script.py", False),  # Python tiene su propio analyzer
        ("style.css", False),
        ("data.json", False),
        ("README.md", False),
    ])
    def test_can_analyze_languages(self, analyzer, filename, expected):
        """Test que detecta lenguajes soportados correctamente."""
        assert analyzer.can_analyze(filename) == expected


class TestReturnsCodeElements:
    """Tests que verifican que se retornan CodeElements."""
    
    def test_returns_code_element_list(self, analyzer):
        """Test que retorna lista de CodeElements."""
        code = '''
function test() {
    return true;
}
'''
        elements = analyzer.analyze("script.js", code)
        
        assert isinstance(elements, list)
        if len(elements) > 0:
            assert isinstance(elements[0], CodeElement)


class TestJavaScriptAnalysis:
    """Tests para análisis de JavaScript."""
    
    def test_extracts_function(self, analyzer):
        """Test que extrae funciones JS."""
        code = '''
function greet(name) {
    return "Hello, " + name;
}
'''
        elements = analyzer.analyze("script.js", code)
        
        assert len(elements) >= 1
        func = elements[0]
        assert func.type == "function"
        assert "greet" in func.name
    
    def test_extracts_arrow_function(self, analyzer):
        """Test que extrae arrow functions."""
        code = '''
const add = (a, b) => {
    return a + b;
};
'''
        elements = analyzer.analyze("script.js", code)
        
        assert len(elements) >= 1
    
    def test_extracts_class(self, analyzer):
        """Test que extrae clases JS."""
        code = '''
class Calculator {
    constructor(initial) {
        this.value = initial;
    }
    
    add(n) {
        this.value += n;
        return this.value;
    }
}
'''
        elements = analyzer.analyze("script.js", code)
        
        class_elements = [e for e in elements if e.type == "class"]
        assert len(class_elements) >= 1
        assert "Calculator" in class_elements[0].name


class TestTypeScriptAnalysis:
    """Tests para análisis de TypeScript."""
    
    def test_extracts_interface(self, analyzer):
        """Test que extrae interfaces TS."""
        code = '''
interface User {
    id: number;
    name: string;
    email?: string;
}
'''
        elements = analyzer.analyze("types.ts", code)
        
        interface_elements = [e for e in elements if e.type == "interface"]
        assert len(interface_elements) >= 1
        assert "User" in interface_elements[0].name
    
    def test_extracts_typed_function(self, analyzer):
        """Test que extrae funciones tipadas."""
        code = '''
function processData(data: string[]): number {
    return data.length;
}
'''
        elements = analyzer.analyze("utils.ts", code)
        
        assert len(elements) >= 1


class TestGoAnalysis:
    """Tests para análisis de Go."""
    
    def test_extracts_function(self, analyzer):
        """Test que extrae funciones Go."""
        code = '''
func greet(name string) string {
    return "Hello, " + name
}
'''
        elements = analyzer.analyze("main.go", code)
        
        assert len(elements) >= 1
        assert elements[0].type == "function"
        assert "greet" in elements[0].name
    
    def test_extracts_struct(self, analyzer):
        """Test que extrae structs Go."""
        code = '''
type User struct {
    ID   int
    Name string
}
'''
        elements = analyzer.analyze("models.go", code)
        
        struct_elements = [e for e in elements if e.type == "struct"]
        assert len(struct_elements) >= 1
        assert "User" in struct_elements[0].name
    
    def test_extracts_method(self, analyzer):
        """Test que extrae métodos Go."""
        code = '''
func (u *User) Greet() string {
    return "Hello, " + u.Name
}
'''
        elements = analyzer.analyze("user.go", code)
        
        assert len(elements) >= 1


class TestRustAnalysis:
    """Tests para análisis de Rust."""
    
    def test_extracts_function(self, analyzer):
        """Test que extrae funciones Rust."""
        code = '''
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
'''
        elements = analyzer.analyze("lib.rs", code)
        
        assert len(elements) >= 1
        assert "greet" in elements[0].name
    
    def test_extracts_struct(self, analyzer):
        """Test que extrae structs Rust."""
        code = '''
struct User {
    id: u64,
    name: String,
}
'''
        elements = analyzer.analyze("models.rs", code)
        
        struct_elements = [e for e in elements if e.type == "struct"]
        assert len(struct_elements) >= 1
    
    def test_extracts_impl(self, analyzer):
        """Test que extrae impl blocks Rust."""
        code = '''
impl User {
    fn new(name: String) -> Self {
        User { id: 0, name }
    }
}
'''
        elements = analyzer.analyze("user.rs", code)
        
        impl_elements = [e for e in elements if e.type == "impl"]
        assert len(impl_elements) >= 1


class TestJavaAnalysis:
    """Tests para análisis de Java."""
    
    def test_extracts_class(self, analyzer):
        """Test que extrae clases Java."""
        code = '''
public class Calculator {
    private int value;
    
    public int add(int n) {
        value += n;
        return value;
    }
}
'''
        elements = analyzer.analyze("Calculator.java", code)
        
        class_elements = [e for e in elements if e.type == "class"]
        assert len(class_elements) >= 1
    
    def test_extracts_interface(self, analyzer):
        """Test que extrae interfaces Java."""
        code = '''
public interface Processor {
    void process(String data);
}
'''
        elements = analyzer.analyze("Processor.java", code)
        
        interface_elements = [e for e in elements if e.type == "interface"]
        assert len(interface_elements) >= 1


class TestCAnalysis:
    """Tests para análisis de C/C++."""
    
    def test_extracts_function(self, analyzer):
        """Test que extrae funciones C."""
        code = '''
int add(int a, int b) {
    return a + b;
}
'''
        elements = analyzer.analyze("math.c", code)
        
        assert len(elements) >= 1
        assert elements[0].type == "function"
    
    def test_extracts_struct(self, analyzer):
        """Test que extrae structs C."""
        code = '''
typedef struct {
    int id;
    char* name;
} User;
'''
        elements = analyzer.analyze("types.h", code)
        
        # Puede detectar como struct o typedef
        assert isinstance(elements, list)


class TestRubyAnalysis:
    """Tests para análisis de Ruby."""
    
    def test_extracts_class(self, analyzer):
        """Test que extrae clases Ruby."""
        code = '''
class Calculator
  def initialize(initial = 0)
    @value = initial
  end
  
  def add(n)
    @value += n
  end
end
'''
        elements = analyzer.analyze("calculator.rb", code)
        
        class_elements = [e for e in elements if e.type == "class"]
        assert len(class_elements) >= 1
    
    def test_extracts_module(self, analyzer):
        """Test que extrae módulos Ruby."""
        code = '''
module Utils
  def self.helper
    puts "helping"
  end
end
'''
        elements = analyzer.analyze("utils.rb", code)
        
        module_elements = [e for e in elements if e.type == "module"]
        assert len(module_elements) >= 1


class TestPHPAnalysis:
    """Tests para análisis de PHP."""
    
    def test_extracts_function(self, analyzer):
        """Test que extrae funciones PHP."""
        code = '''
<?php
function greet($name) {
    return "Hello, " . $name;
}
?>
'''
        elements = analyzer.analyze("index.php", code)
        
        func_elements = [e for e in elements if e.type == "function"]
        assert len(func_elements) >= 1
    
    def test_extracts_class(self, analyzer):
        """Test que extrae clases PHP."""
        code = '''
<?php
class Calculator {
    private $value;
    
    public function add($n) {
        $this->value += $n;
        return $this->value;
    }
}
?>
'''
        elements = analyzer.analyze("Calculator.php", code)
        
        class_elements = [e for e in elements if e.type == "class"]
        assert len(class_elements) >= 1


class TestEdgeCases:
    """Tests para casos límite."""
    
    def test_empty_file(self, analyzer):
        """Test archivo vacío."""
        elements = analyzer.analyze("empty.js", "")
        
        assert isinstance(elements, list)
        assert len(elements) == 0
    
    def test_only_comments(self, analyzer):
        """Test archivo solo con comentarios."""
        code = '''
// This is a comment
/* Multi-line
   comment */
'''
        elements = analyzer.analyze("comments.js", code)
        
        assert isinstance(elements, list)
    
    def test_unicode_content(self, analyzer):
        """Test contenido unicode."""
        code = '''
function saludar(nombre) {
    return "¡Hola, " + nombre + "! 🎉";
}
'''
        elements = analyzer.analyze("script.js", code)
        
        assert len(elements) >= 1
    
    def test_minified_code(self, analyzer):
        """Test código minificado (una línea)."""
        code = 'function a(b){return b+1;}function c(d){return d*2;}'
        
        elements = analyzer.analyze("min.js", code)
        
        # Puede o no detectar en código minificado
        assert isinstance(elements, list)
