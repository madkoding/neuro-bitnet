# =============================================================================
# neuro-bitnet Dockerfile
# Multi-model support: Falcon3-7B-Instruct-1.58bit / BitNet-b1.58-2B-4T
# Build with: docker build --build-arg MODEL_VARIANT=falcon-7b .
# =============================================================================

# Argumento para seleccionar modelo (falcon-7b o bitnet-2b)
ARG MODEL_VARIANT=falcon-7b

FROM nvidia/cuda:12.6.3-devel-ubuntu22.04 AS builder

# Re-declarar ARG después de FROM
ARG MODEL_VARIANT

# Evitar prompts interactivos
ENV DEBIAN_FRONTEND=noninteractive

# Instalar dependencias de compilación
RUN apt-get update && apt-get install -y \
    git \
    cmake \
    ninja-build \
    build-essential \
    python3 \
    python3-pip \
    python3-venv \
    wget \
    curl \
    software-properties-common \
    gnupg \
    lsb-release \
    && rm -rf /var/lib/apt/lists/*

# Instalar Clang 18 (requerido por BitNet)
RUN wget https://apt.llvm.org/llvm.sh && \
    chmod +x llvm.sh && \
    ./llvm.sh 18 && \
    rm llvm.sh

# Crear enlaces simbólicos para clang y clang++ (requerido por setup_env.py)
RUN ln -sf /usr/bin/clang-18 /usr/bin/clang && \
    ln -sf /usr/bin/clang++-18 /usr/bin/clang++

# Configurar Clang 18 como compilador por defecto
ENV CC=clang-18
ENV CXX=clang++-18

# Crear directorio de trabajo
WORKDIR /build

# Clonar BitNet con submodules
RUN git clone --recursive https://github.com/microsoft/BitNet.git

WORKDIR /build/BitNet

# Instalar dependencias Python
RUN pip3 install --no-cache-dir -r requirements.txt

# Instalar huggingface-cli para descargar modelos
RUN pip3 install --no-cache-dir huggingface_hub[cli]

# =============================================================================
# Descargar y convertir modelo según MODEL_VARIANT
# =============================================================================

# Copiar script de descarga de modelos
COPY scripts/download_model.sh /build/BitNet/
RUN chmod +x /build/BitNet/download_model.sh

# Ejecutar descarga según variante
RUN /build/BitNet/download_model.sh ${MODEL_VARIANT}

# =============================================================================
# Imagen de producción
# =============================================================================

FROM nvidia/cuda:12.6.3-runtime-ubuntu22.04

# Re-declarar ARG para la etapa de producción
ARG MODEL_VARIANT

ENV DEBIAN_FRONTEND=noninteractive

# Instalar dependencias de runtime
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    curl \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Instalar huggingface-cli (para descarga de modelos en runtime si es necesario)
RUN pip3 install --no-cache-dir huggingface_hub[cli]

# Crear usuario no-root
RUN useradd -m -s /bin/bash bitnet

# Crear directorios
RUN mkdir -p /app/models /app/scripts /app/build

# Copiar binarios compilados desde builder
COPY --from=builder /build/BitNet/build /app/build
COPY --from=builder /build/BitNet/models /app/models

# Copiar librerías necesarias de llama.cpp
COPY --from=builder /build/BitNet/build/3rdparty/llama.cpp/src/*.so* /app/lib/ 
COPY --from=builder /build/BitNet/build/3rdparty/llama.cpp/ggml/src/*.so* /app/lib/

# Copiar scripts
COPY scripts/ /app/scripts/
RUN chmod +x /app/scripts/*.sh

# Configurar LD_LIBRARY_PATH para las librerías de bitnet.cpp
ENV LD_LIBRARY_PATH=/app/lib:/app/build/3rdparty/llama.cpp/src:/app/build/3rdparty/llama.cpp/ggml/src:$LD_LIBRARY_PATH

# Variables de entorno por defecto
ENV BITNET_HOST=0.0.0.0
ENV BITNET_PORT=8080
ENV BITNET_CTX_SIZE=4096
ENV BITNET_PARALLEL=4
ENV BITNET_GPU_LAYERS=99
ENV BITNET_THREADS=4

# Configurar MODEL_VARIANT - el entrypoint buscará el GGUF automáticamente
# falcon-7b -> /app/models/falcon-7b/falcon3-7b-instruct-1.58bit.gguf
# bitnet-2b -> /app/models/bitnet-2b/ggml-model-i2_s.gguf
ENV MODEL_VARIANT=${MODEL_VARIANT}

# Exponer puerto
EXPOSE 8080

# Healthcheck
HEALTHCHECK --interval=30s --timeout=15s --start-period=180s --retries=3 \
    CMD /app/scripts/healthcheck.sh

WORKDIR /app

# Cambiar a usuario no-root
USER bitnet

# Comando de inicio
ENTRYPOINT ["/app/scripts/entrypoint.sh"]
