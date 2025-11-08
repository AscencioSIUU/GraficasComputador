#!/bin/bash
# Script optimizado para ejecutar el diorama con máximo rendimiento

echo "=========================================="
echo "  Raytracing Diorama - Minecraft Style"
echo "=========================================="
echo ""
echo "Configuración de rendimiento:"
echo "- 8 threads de Rayon"
echo "- Pantalla completa"
echo "- Resolución inicial: 0.33x (para velocidad)"
echo "- Sin reflexión/refracción iniciales"
echo ""
echo "Controles rápidos:"
echo "  1/2/3/4 - Cambiar resolución (4=más rápido)"
echo "  F2/F3   - Toggle efectos avanzados"
echo "  ESC     - Salir de fullscreen"
echo ""

# Compilar en release con optimizaciones nativas
echo "Compilando con optimizaciones..."
RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "Ejecutando..."
    echo ""
    ./target/release/raytracing_diorama
else
    echo "Error en la compilación"
    exit 1
fi
