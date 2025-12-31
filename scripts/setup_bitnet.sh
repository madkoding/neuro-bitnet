#!/bin/bash
# Setup script for bitnet.cpp - Microsoft's BitNet inference runtime
# 
# This script downloads and compiles bitnet.cpp which is required for
# local inference with BitNet 1.58-bit models.

set -e

BITNET_DIR="${BITNET_DIR:-$HOME/.local/share/bitnet.cpp}"
BITNET_REPO="https://github.com/microsoft/BitNet.git"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check for required tools
check_requirements() {
    info "Checking requirements..."
    
    # Check for git
    if ! command -v git &> /dev/null; then
        error "git is required but not installed. Please install git first."
    fi
    
    # Check for cmake
    if ! command -v cmake &> /dev/null; then
        error "cmake is required but not installed. Please install cmake first."
    fi
    CMAKE_VERSION=$(cmake --version | head -n1 | cut -d' ' -f3)
    info "Found cmake $CMAKE_VERSION"
    
    # Check for clang >= 18
    CLANG_CMD=""
    for cmd in clang-18 clang-19 clang-20 clang; do
        if command -v $cmd &> /dev/null; then
            VERSION=$($cmd --version | head -n1 | grep -oP '\d+' | head -n1)
            if [ "$VERSION" -ge 18 ]; then
                CLANG_CMD=$cmd
                info "Found $cmd (version $VERSION)"
                break
            fi
        fi
    done
    
    if [ -z "$CLANG_CMD" ]; then
        error "clang >= 18 is required but not found. Please install clang-18 or newer."
    fi
    
    # Check for python3
    if ! command -v python3 &> /dev/null; then
        error "python3 is required but not installed."
    fi
    PYTHON_VERSION=$(python3 --version 2>&1 | cut -d' ' -f2)
    info "Found Python $PYTHON_VERSION"
    
    success "All requirements satisfied"
}

# Install dependencies based on OS
install_dependencies() {
    info "Detecting OS..."
    
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
    elif [ -f /etc/debian_version ]; then
        OS="debian"
    elif [ -f /etc/arch-release ]; then
        OS="arch"
    else
        OS=$(uname -s)
    fi
    
    info "Detected OS: $OS"
    
    case $OS in
        ubuntu|debian)
            info "Installing dependencies with apt..."
            sudo apt-get update
            sudo apt-get install -y git cmake clang-18 python3 python3-pip build-essential
            ;;
        arch|manjaro)
            info "Installing dependencies with pacman..."
            sudo pacman -Sy --needed git cmake clang python python-pip base-devel
            ;;
        fedora)
            info "Installing dependencies with dnf..."
            sudo dnf install -y git cmake clang python3 python3-pip gcc-c++
            ;;
        darwin|Darwin)
            info "Installing dependencies with brew..."
            brew install git cmake llvm python3
            export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
            ;;
        *)
            warn "Unknown OS: $OS. Please install git, cmake, clang>=18, and python3 manually."
            ;;
    esac
}

# Clone and build bitnet.cpp
build_bitnet() {
    info "Setting up bitnet.cpp..."
    
    # Clone or update repository
    if [ -d "$BITNET_DIR" ]; then
        info "Updating existing bitnet.cpp..."
        cd "$BITNET_DIR"
        git fetch
        git pull
    else
        info "Cloning bitnet.cpp..."
        git clone --recursive "$BITNET_REPO" "$BITNET_DIR"
        cd "$BITNET_DIR"
    fi
    
    # Create build directory
    BUILD_DIR="$BITNET_DIR/build"
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    
    # Configure with cmake
    info "Configuring build..."
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_C_COMPILER=$(which clang-18 || which clang) \
        -DCMAKE_CXX_COMPILER=$(which clang++-18 || which clang++)
    
    # Build
    info "Building bitnet.cpp (this may take a few minutes)..."
    cmake --build . --config Release -j$(nproc)
    
    success "Build complete!"
}

# Install the binary
install_binary() {
    info "Installing llama-cli binary..."
    
    BINARY_PATH="$BITNET_DIR/build/bin/llama-cli"
    
    if [ ! -f "$BINARY_PATH" ]; then
        error "Build failed: llama-cli binary not found at $BINARY_PATH"
    fi
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Copy binary with special name to avoid conflict with regular llama.cpp
    cp "$BINARY_PATH" "$INSTALL_DIR/llama-cli-bitnet"
    chmod +x "$INSTALL_DIR/llama-cli-bitnet"
    
    success "Installed to: $INSTALL_DIR/llama-cli-bitnet"
    
    # Check if directory is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add this line to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "Or set BITNET_CLI_PATH environment variable:"
        echo "  export BITNET_CLI_PATH=\"$INSTALL_DIR/llama-cli-bitnet\""
    fi
}

# Verify installation
verify_installation() {
    info "Verifying installation..."
    
    BINARY="$INSTALL_DIR/llama-cli-bitnet"
    
    if [ -x "$BINARY" ]; then
        VERSION=$($BINARY --version 2>&1 || echo "version unknown")
        success "bitnet.cpp installed successfully!"
        echo ""
        echo "Binary: $BINARY"
        echo "Version: $VERSION"
        echo ""
        echo "Usage with neuro-bitnet:"
        echo "  export BITNET_CLI_PATH=\"$BINARY\""
        echo "  neuro ask \"What is BitNet?\""
    else
        error "Installation verification failed"
    fi
}

# Print help
print_help() {
    echo "BitNet.cpp Setup Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --install-deps    Install system dependencies"
    echo "  --build           Clone and build bitnet.cpp"
    echo "  --install         Install the binary to $INSTALL_DIR"
    echo "  --all             Do all of the above (default)"
    echo "  --check           Only check requirements"
    echo "  --help            Show this help"
    echo ""
    echo "Environment variables:"
    echo "  BITNET_DIR        Directory for bitnet.cpp source (default: ~/.local/share/bitnet.cpp)"
    echo "  INSTALL_DIR       Directory for binary (default: ~/.local/bin)"
    echo ""
}

# Main
main() {
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                   BitNet.cpp Setup Script                    ║"
    echo "║              Microsoft's 1-bit LLM Inference                 ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    
    case "${1:-all}" in
        --help|-h)
            print_help
            ;;
        --check)
            check_requirements
            ;;
        --install-deps)
            install_dependencies
            ;;
        --build)
            check_requirements
            build_bitnet
            ;;
        --install)
            install_binary
            verify_installation
            ;;
        --all|*)
            check_requirements
            build_bitnet
            install_binary
            verify_installation
            ;;
    esac
}

main "$@"
