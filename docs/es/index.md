---
layout: default
title: Inicio
nav_order: 1
lang: es
---

# neuro-bitnet

[![CI](https://github.com/madkoding/neuro-bitnet/actions/workflows/ci.yml/badge.svg)](https://github.com/madkoding/neuro-bitnet/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/neuro-cli.svg)](LICENSE-MIT)

🌐 Español | **[English](../)**

Un servidor **RAG (Retrieval Augmented Generation)** de alto rendimiento escrito en Rust con soporte de inferencia local **BitNet 1.58-bit**.

## ✨ Características

- 🚀 **Alto Rendimiento** - Rust nativo con operaciones vectoriales optimizadas con SIMD
- 🧠 **Inferencia BitNet** - Inferencia local solo-CPU con modelos 1.58-bit de Microsoft
- 📊 **Embeddings Nativos** - Modelos de embedding integrados via fastembed
- 🔍 **Búsqueda Semántica** - Búsqueda rápida por similitud coseno
- 🌐 **Búsqueda Web** - Integración con Wikipedia para aumentar conocimiento
- 📦 **Binario Único** - Compilación estática, sin dependencias en runtime

## 🚀 Inicio Rápido

### Instalación

```bash
# Desde releases
curl -L https://github.com/madkoding/neuro-bitnet/releases/latest/download/neuro-linux-x86_64 -o neuro
chmod +x neuro
sudo mv neuro /usr/local/bin/

# Desde código fuente
cargo install neuro-cli
```

### Configurar BitNet (para inferencia local)

```bash
# Compilar bitnet.cpp
./scripts/setup_bitnet.sh

# Descargar un modelo BitNet
neuro model download 2b

# Hacer preguntas localmente
neuro ask "¿Cuál es la capital de Francia?"
```

## 📊 Resultados del Benchmark BitNet

| Métrica | BitNet b1.58 2B-4T |
|---------|-------------------|
| **Tasa de Éxito** | 100% |
| **Tamaño del Modelo** | 1.1 GB |
| **Respuesta Promedio** | 2.8s |
| **Backend** | Solo-CPU |

[Ver informe completo de benchmark →](benchmarks)

## 📚 Documentación

- [Guía de Inferencia Local](local-inference) - Configurar BitNet para inferencia local
- [Benchmarks](benchmarks) - Comparación de rendimiento y resultados de tests
- [Referencia de API](api) - Documentación de la API HTTP

## 🏗️ Arquitectura

```
┌─────────────────┐
│   neuro-cli     │  Interfaz CLI
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌───────┐ ┌───────┐
│  RAG  │ │BitNet │  Inferencia
└───┬───┘ └───┬───┘
    │         │
    ▼         ▼
┌───────┐ ┌───────┐
│Storage│ │ GGUF  │  Modelos
└───────┘ └───────┘
```

## 📜 Licencia

Licenciado bajo MIT o Apache 2.0 a tu elección.

---

[GitHub](https://github.com/madkoding/neuro-bitnet) · [Releases](https://github.com/madkoding/neuro-bitnet/releases) · [Issues](https://github.com/madkoding/neuro-bitnet/issues)
