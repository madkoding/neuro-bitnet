"""
Pytest Configuration and Fixtures
=================================

Fixtures compartidos para todos los tests.
"""

import sys
import tempfile
from pathlib import Path
from typing import Generator

import pytest

# Añadir src al path para imports
sys.path.insert(0, str(Path(__file__).parent.parent))


@pytest.fixture
def temp_dir() -> Generator[Path, None, None]:
    """Directorio temporal para tests."""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield Path(tmpdir)


@pytest.fixture
def sample_python_code() -> str:
    """Código Python de ejemplo para tests."""
    return '''
"""Sample module for testing."""

import os
from typing import List

class Calculator:
    """A simple calculator class."""
    
    def __init__(self, initial: int = 0):
        self.value = initial
    
    def add(self, x: int) -> int:
        """Add x to the current value."""
        self.value += x
        return self.value
    
    def subtract(self, x: int) -> int:
        """Subtract x from the current value."""
        self.value -= x
        return self.value


def factorial(n: int) -> int:
    """Calculate factorial of n."""
    if n <= 1:
        return 1
    return n * factorial(n - 1)


async def fetch_data(url: str) -> dict:
    """Async function to fetch data."""
    return {"url": url}
'''


@pytest.fixture
def sample_javascript_code() -> str:
    """Código JavaScript de ejemplo para tests."""
    return '''
// Sample JavaScript module

export class UserService {
    constructor(apiUrl) {
        this.apiUrl = apiUrl;
    }
    
    async getUser(id) {
        const response = await fetch(`${this.apiUrl}/users/${id}`);
        return response.json();
    }
}

export function formatDate(date) {
    return date.toISOString().split('T')[0];
}

const helper = (x) => x * 2;
'''


@pytest.fixture
def sample_documents() -> list:
    """Lista de documentos de ejemplo."""
    return [
        {
            "content": "Python es un lenguaje de programación de alto nivel.",
            "source": "manual",
            "user_id": "test_user"
        },
        {
            "content": "La capital de Francia es París.",
            "source": "manual", 
            "user_id": "test_user"
        },
        {
            "content": "Docker permite crear contenedores para aplicaciones.",
            "source": "file",
            "user_id": "test_user"
        }
    ]


@pytest.fixture
def classification_test_cases() -> list:
    """Casos de test para el clasificador."""
    return [
        # (query, expected_category)
        ("25+17", "math"),
        ("¿Cuánto es 100/4?", "math"),
        ("Escribe una función que sume dos números", "code"),
        ("Hola mundo en Python", "code"),
        ("¿Cuál es la capital de Francia?", "factual"),
        ("¿Quién fue Albert Einstein?", "factual"),
        ("Hola, ¿cómo estás?", "greeting"),
        ("Buenos días", "greeting"),
        ("Si llueve, entonces se moja", "reasoning"),
        ("¿Qué sigue después de 2, 4, 6?", "reasoning"),
        ("Traduce esto al inglés", "tools"),
        ("¿Cuál es el clima en Madrid?", "tools"),
    ]
