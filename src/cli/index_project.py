#!/usr/bin/env python3
"""
Project Indexer - Indexador de Proyectos para RAG
=================================================

Lee un proyecto completo, genera documentación automática y la indexa en el RAG.

Uso:
    python -m src.cli.index_project /ruta/al/proyecto
    python -m src.cli.index_project /ruta/al/proyecto --rag-url http://localhost:11436
"""

import os
import argparse
from pathlib import Path
from typing import List, Dict

import requests

from src.rag.core import DEFAULT_RAG_PORT
from src.rag.indexer import PythonAnalyzer, GenericAnalyzer, CodeElement


# Extensiones de código soportadas
CODE_EXTENSIONS = {
    '.py': 'python',
    '.js': 'javascript', 
    '.ts': 'typescript',
    '.jsx': 'javascript',
    '.tsx': 'typescript',
    '.go': 'go',
    '.rs': 'rust',
    '.java': 'java',
    '.c': 'c',
    '.cpp': 'cpp',
    '.h': 'c',
    '.hpp': 'cpp',
    '.rb': 'ruby',
    '.php': 'php',
    '.sh': 'bash',
    '.yaml': 'yaml',
    '.yml': 'yaml',
    '.json': 'json',
    '.toml': 'toml',
    '.md': 'markdown',
    '.sql': 'sql',
}

# Archivos importantes a incluir siempre
IMPORTANT_FILES = [
    'README.md', 'README.rst', 'README.txt', 'README',
    'CHANGELOG.md', 'CHANGELOG', 'HISTORY.md',
    'CONTRIBUTING.md', 'CONTRIBUTING',
    'LICENSE', 'LICENSE.md', 'LICENSE.txt',
    'setup.py', 'setup.cfg', 'pyproject.toml',
    'package.json', 'tsconfig.json',
    'Cargo.toml', 'go.mod',
    'Dockerfile', 'docker-compose.yml', 'docker-compose.yaml',
    'Makefile', 'CMakeLists.txt',
    '.env.example', '.env.sample',
    'requirements.txt', 'requirements-dev.txt',
]

# Directorios a ignorar
IGNORE_DIRS = {
    '.git', '.svn', '.hg',
    'node_modules', '__pycache__', '.pytest_cache',
    'venv', 'env', '.venv', '.env',
    'dist', 'build', 'target', 'out',
    '.idea', '.vscode', '.vs',
    'coverage', '.coverage', 'htmlcov',
    '.tox', '.nox', '.mypy_cache',
    'eggs', '*.egg-info',
}


class ProjectIndexer:
    """Indexa un proyecto completo en el RAG."""
    
    def __init__(self, rag_url: str):
        self.rag_url = rag_url
        self.python_analyzer = PythonAnalyzer()
        self.generic_analyzer = GenericAnalyzer()
        self.indexed_count = 0
        self.files_processed = 0
    
    def index_project(self, project_path_str: str) -> Dict:
        """Indexa todo el proyecto."""
        project_path = Path(project_path_str).resolve()
        project_name = project_path.name
        
        print(f"📁 Indexando proyecto: {project_name}")
        print(f"📂 Ruta: {project_path}")
        print("=" * 60)
        
        # 1. Generar estructura del proyecto
        structure = self._get_project_structure(project_path)
        self._index_document(
            f"Estructura del proyecto {project_name}:\n{structure}",
            f"{project_name}/structure",
            "project_structure"
        )
        
        # 2. Indexar archivos importantes
        print("\n📄 Indexando archivos importantes...")
        for filename in IMPORTANT_FILES:
            file_path = project_path / filename
            if file_path.exists():
                self._index_file(file_path, project_name)
        
        # 3. Analizar y indexar código
        print("\n🔍 Analizando código fuente...")
        all_elements: List[CodeElement] = []
        
        for file_path in self._iter_code_files(project_path):
            elements = self._analyze_file(file_path)
            all_elements.extend(elements)
            self.files_processed += 1
            
            # Indexar resumen del archivo
            self._index_code_file(file_path, project_name)
        
        # 4. Generar e indexar documentación automática
        print("\n📝 Generando documentación automática...")
        self._generate_and_index_docs(project_name, all_elements)
        
        # 5. Generar índice de API
        print("\n🔧 Generando índice de API...")
        self._generate_api_index(project_name, all_elements)
        
        print("\n" + "=" * 60)
        print(f"✅ Indexación completada!")
        print(f"   📁 Archivos procesados: {self.files_processed}")
        print(f"   📄 Documentos indexados: {self.indexed_count}")
        
        return {
            "project": project_name,
            "files_processed": self.files_processed,
            "documents_indexed": self.indexed_count,
            "elements_found": len(all_elements)
        }
    
    def _get_project_structure(self, project_path: Path, max_depth: int = 3) -> str:
        """Genera representación de la estructura del proyecto."""
        lines = []
        
        def walk(path: Path, prefix: str = "", depth: int = 0):
            if depth > max_depth:
                return
            
            try:
                items = sorted(path.iterdir(), key=lambda x: (x.is_file(), x.name))
            except PermissionError:
                return
            
            for i, item in enumerate(items):
                if item.name in IGNORE_DIRS or item.name.startswith('.'):
                    continue
                
                is_last = i == len(items) - 1
                current_prefix = "└── " if is_last else "├── "
                lines.append(f"{prefix}{current_prefix}{item.name}")
                
                if item.is_dir():
                    next_prefix = prefix + ("    " if is_last else "│   ")
                    walk(item, next_prefix, depth + 1)
        
        lines.append(project_path.name + "/")
        walk(project_path)
        return "\n".join(lines[:100])
    
    def _iter_code_files(self, project_path: Path):
        """Itera sobre archivos de código del proyecto."""
        for root, dirs, files in os.walk(project_path):
            # Filtrar directorios ignorados
            dirs[:] = [d for d in dirs if d not in IGNORE_DIRS and not d.startswith('.')]
            
            for filename in files:
                ext = Path(filename).suffix.lower()
                if ext in CODE_EXTENSIONS:
                    yield Path(root) / filename
    
    def _analyze_file(self, file_path: Path) -> List[CodeElement]:
        """Analiza un archivo de código."""
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            ext = file_path.suffix.lower()
            language = CODE_EXTENSIONS.get(ext, 'unknown')
            
            if language == 'python':
                return self.python_analyzer.analyze(str(file_path), content)
            else:
                return self.generic_analyzer.analyze(str(file_path), content, language)
        except Exception as e:
            print(f"   ⚠️  Error analizando {file_path}: {e}")
            return []
    
    def _index_file(self, file_path: Path, project_name: str):
        """Indexa un archivo completo."""
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            relative_path = file_path.name
            
            # Limitar tamaño
            if len(content) > 10000:
                content = content[:10000] + "\n... (truncado)"
            
            self._index_document(
                f"Archivo: {relative_path}\n\n{content}",
                f"{project_name}/{relative_path}",
                "file"
            )
            print(f"   ✓ {relative_path}")
        except Exception as e:
            print(f"   ⚠️  Error leyendo {file_path}: {e}")
    
    def _index_code_file(self, file_path: Path, project_name: str):
        """Indexa un archivo de código con resumen."""
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            relative_path = str(file_path).split(project_name)[-1].lstrip('/')
            
            # Crear resumen del archivo
            lines = content.split('\n')
            imports = [
                l for l in lines[:50] 
                if l.strip().startswith(('import ', 'from ', 'require', '#include'))
            ]
            
            summary = f"""Archivo: {relative_path}
Lenguaje: {CODE_EXTENSIONS.get(file_path.suffix.lower(), 'unknown')}
Líneas: {len(lines)}
Imports/Includes: {len(imports)}

Primeras líneas:
{chr(10).join(lines[:30])}
"""
            
            self._index_document(summary, f"{project_name}/{relative_path}", "code_file")
        except Exception:
            pass
    
    def _generate_and_index_docs(self, project_name: str, elements: List[CodeElement]):
        """Genera documentación automática para funciones y clases."""
        # Agrupar por archivo
        by_file: Dict[str, List[CodeElement]] = {}
        for elem in elements:
            if elem.file_path not in by_file:
                by_file[elem.file_path] = []
            by_file[elem.file_path].append(elem)
        
        for file_path, file_elements in by_file.items():
            doc_parts = [f"# Documentación: {Path(file_path).name}\n"]
            
            classes = [e for e in file_elements if e.type == 'class']
            functions = [e for e in file_elements if e.type == 'function']
            methods = [e for e in file_elements if e.type == 'method']
            
            if classes:
                doc_parts.append("\n## Clases\n")
                for cls in classes:
                    doc_parts.append(f"### {cls.name}\n")
                    doc_parts.append(f"```\n{cls.signature}\n```\n")
                    if cls.docstring:
                        doc_parts.append(f"{cls.docstring}\n")
            
            if functions:
                doc_parts.append("\n## Funciones\n")
                for func in functions:
                    doc_parts.append(f"### {func.name}\n")
                    doc_parts.append(f"```\n{func.signature}\n```\n")
                    if func.docstring:
                        doc_parts.append(f"{func.docstring}\n")
            
            if methods:
                doc_parts.append("\n## Métodos\n")
                for method in methods:
                    doc_parts.append(f"### {method.name}\n")
                    doc_parts.append(f"```\n{method.signature}\n```\n")
                    if method.docstring:
                        doc_parts.append(f"{method.docstring}\n")
            
            doc = "\n".join(doc_parts)
            self._index_document(
                doc, 
                f"{project_name}/docs/{Path(file_path).name}", 
                "auto_doc"
            )
    
    def _generate_api_index(self, project_name: str, elements: List[CodeElement]):
        """Genera índice de API del proyecto."""
        classes = [e for e in elements if e.type == 'class']
        functions = [e for e in elements if e.type == 'function']
        
        index = f"""# API Index: {project_name}

## Clases ({len(classes)})
"""
        for cls in classes[:50]:
            index += f"- **{cls.name}** ({Path(cls.file_path).name}:{cls.line_number})\n"
        
        index += f"\n## Funciones ({len(functions)})\n"
        for func in functions[:50]:
            index += f"- **{func.name}** ({Path(func.file_path).name}:{func.line_number})\n"
        
        self._index_document(index, f"{project_name}/api_index", "api_index")
    
    def _index_document(self, content: str, source: str, doc_type: str):
        """Envía documento al RAG server."""
        try:
            response = requests.post(
                f"{self.rag_url}/add",
                json={
                    "content": content,
                    "source": source,
                    "metadata": {"type": doc_type}
                },
                timeout=30
            )
            if response.status_code == 200:
                self.indexed_count += 1
            else:
                print(f"   ⚠️  Error indexando {source}: {response.text}")
        except requests.exceptions.ConnectionError:
            print(f"   ❌ RAG server no disponible en {self.rag_url}")
        except Exception as e:
            print(f"   ⚠️  Error: {e}")


def main():
    parser = argparse.ArgumentParser(
        description="Indexa un proyecto en el RAG para documentación automática"
    )
    parser.add_argument(
        "project_path",
        help="Ruta al proyecto a indexar"
    )
    parser.add_argument(
        "--rag-url",
        default=f"http://localhost:{DEFAULT_RAG_PORT}",
        help=f"URL del RAG server (default: http://localhost:{DEFAULT_RAG_PORT})"
    )
    
    args = parser.parse_args()
    
    # Verificar que el proyecto existe
    project_path = Path(args.project_path)
    if not project_path.exists():
        print(f"❌ Error: No existe el directorio {project_path}")
        return 1
    
    # Verificar RAG server
    try:
        r = requests.get(f"{args.rag_url}/health", timeout=5)
        if r.status_code != 200:
            print(f"❌ RAG server no responde correctamente")
            return 1
    except requests.exceptions.ConnectionError:
        print(f"❌ No se puede conectar al RAG server en {args.rag_url}")
        print("   Asegúrate de que el servidor está corriendo:")
        print(f"   python -m src.server.rag_server")
        return 1
    
    # Indexar proyecto
    indexer = ProjectIndexer(args.rag_url)
    result = indexer.index_project(str(project_path))
    
    print(f"\n💡 Ahora puedes hacer preguntas sobre el proyecto:")
    print(f'   python -m src.cli.rag_client "¿Cómo funciona la clase X?"')
    print(f'   python -m src.cli.rag_client "¿Qué hace la función Y?"')
    
    return 0


if __name__ == "__main__":
    exit(main())
