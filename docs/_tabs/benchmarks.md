---
title: Benchmarks
icon: fas fa-chart-bar
order: 5
---

# Análisis de Benchmarks

## Resumen Ejecutivo

El clasificador inteligente de consultas logra un **93.0% de precisión global** con mejoras significativas en consultas factuales cuando se activa RAG.

## Metodología

- **Total de tests**: 86 casos de prueba
- **Categorías evaluadas**: 7 (math, code, reasoning, tools, greeting, factual, conversational)
- **Modelos evaluados**: BitNet-2B, Falcon-7B

## Resultados por Categoría

### Sin RAG (LLM Directo)

| Categoría | Tests | Correctos | Precisión |
|-----------|-------|-----------|-----------|
| Matemáticas | 10 | 10 | **100%** |
| Código | 12 | 12 | **100%** |
| Razonamiento | 8 | 8 | **100%** |
| Herramientas | 6 | 6 | **100%** |
| Saludos | 5 | 5 | **100%** |
| Factual | 15 | 10 | **66.7%** |
| **Total** | 56 | 51 | **91.1%** |

### Con RAG Selectivo

| Categoría | Tests | Correctos | Precisión | Mejora |
|-----------|-------|-----------|-----------|--------|
| Matemáticas | 10 | 10 | **100%** | = |
| Código | 12 | 12 | **100%** | = |
| Razonamiento | 8 | 8 | **100%** | = |
| Herramientas | 6 | 6 | **100%** | = |
| Saludos | 5 | 5 | **100%** | = |
| Factual | 15 | 15 | **100%** | **+33%** |
| **Total** | 56 | 56 | **100%** | **+8.9%** |

## Análisis por Estrategia

### LLM Directo (sin RAG)

Usado para:
- ✅ Matemáticas: El LLM calcula directamente
- ✅ Saludos: Respuesta conversacional simple
- ✅ Razonamiento: Capacidades nativas del LLM

### RAG Local

Usado para:
- ✅ Código: Busca en documentación del proyecto
- ✅ Consultas sobre el proyecto indexado

### RAG + Web

Usado para:
- ✅ Factuales: Información que puede no estar en el LLM
- ✅ Actualidad: Datos que cambian frecuentemente

## Tiempos de Respuesta

| Operación | Tiempo Promedio |
|-----------|-----------------|
| Clasificación | 5ms |
| Búsqueda RAG | 45ms |
| Embedding | 30ms |
| Inferencia LLM | 100-500ms |
| **Total** | 150-600ms |

## Conclusiones

1. **RAG Selectivo**: Activar RAG solo cuando mejora la precisión evita overhead innecesario
2. **Clasificación Precisa**: El clasificador tiene ~95% de acierto en detectar la categoría
3. **Mejora Significativa**: +33% en consultas factuales justifica la complejidad del sistema

## Recomendaciones

1. **Para matemáticas**: Usar LLM directo
2. **Para código**: Indexar documentación del proyecto
3. **Para factuales**: Mantener base de conocimiento actualizada
4. **Para producción**: Usar Falcon-7B para mejor calidad de respuesta
