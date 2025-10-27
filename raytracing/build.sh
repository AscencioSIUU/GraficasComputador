#!/bin/bash

# Script de ayuda para compilar y ejecutar el proyecto

echo "==================================="
echo "  Raytracing Diorama - Build Tool"
echo "==================================="
echo ""

case "$1" in
    "run")
        echo "🚀 Ejecutando en modo debug..."
        cargo run
        ;;
    "release")
        echo "🚀 Ejecutando en modo release (optimizado)..."
        cargo run --release
        ;;
    "build")
        echo "🔨 Compilando en modo release..."
        cargo build --release
        ;;
    "clean")
        echo "🧹 Limpiando archivos de compilación..."
        cargo clean
        ;;
    "check")
        echo "✅ Verificando código..."
        cargo check
        ;;
    "help"|*)
        echo "Uso: ./build.sh [comando]"
        echo ""
        echo "Comandos disponibles:"
        echo "  run      - Ejecutar en modo debug"
        echo "  release  - Ejecutar en modo release (mejor FPS)"
        echo "  build    - Compilar en modo release"
        echo "  clean    - Limpiar archivos de compilación"
        echo "  check    - Verificar el código"
        echo "  help     - Mostrar esta ayuda"
        echo ""
        echo "Ejemplo: ./build.sh release"
        ;;
esac
