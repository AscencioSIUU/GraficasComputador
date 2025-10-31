#!/bin/bash

# Script para preparar el proyecto para entrega
# Uso: bash preparar_entrega.sh

echo "🚀 Preparando Planet Shaders para entrega..."
echo ""

# Verificar que estamos en el directorio correcto
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Este script debe ejecutarse desde la carpeta planet_shaders/"
    exit 1
fi

echo "✅ Directorio correcto verificado"
echo ""

# Verificar que existe sphere.obj
if [ ! -f "assets/sphere.obj" ]; then
    echo "⚠️  Advertencia: No se encontró assets/sphere.obj"
    echo "   Buscando sphere.obj en el proyecto..."
    
    if [ -f "static_shaders/sphere.obj" ]; then
        cp static_shaders/sphere.obj assets/sphere.obj
        echo "✅ sphere.obj copiado a assets/"
    elif [ -f "../static_shaders/sphere.obj" ]; then
        cp ../static_shaders/sphere.obj assets/sphere.obj
        echo "✅ sphere.obj copiado a assets/"
    else
        echo "❌ Error: No se encontró sphere.obj en ninguna ubicación"
        echo "   Por favor, coloca sphere.obj en assets/ manualmente"
        exit 1
    fi
fi

echo "✅ Modelo sphere.obj encontrado"
echo ""

# Compilar en modo release
echo "🔨 Compilando proyecto..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Error en la compilación"
    exit 1
fi

echo "✅ Compilación exitosa"
echo ""

# Verificar carpeta de screenshots
mkdir -p screenshots
echo "✅ Carpeta screenshots/ creada"
echo ""

# Verificar capturas de pantalla
echo "📸 Verificando capturas de pantalla..."
missing_screenshots=false

if [ ! -f "screenshots/rocky_planet.png" ]; then
    echo "⚠️  Falta: screenshots/rocky_planet.png"
    missing_screenshots=true
fi

if [ ! -f "screenshots/gas_giant.png" ]; then
    echo "⚠️  Falta: screenshots/gas_giant.png"
    missing_screenshots=true
fi

if [ ! -f "screenshots/lava_planet.png" ]; then
    echo "⚠️  Falta: screenshots/lava_planet.png"
    missing_screenshots=true
fi

if [ "$missing_screenshots" = true ]; then
    echo ""
    echo "📷 Instrucciones para tomar capturas:"
    echo "   1. Ejecuta: cargo run --release"
    echo "   2. Presiona 1 → Toma captura → Guarda como: screenshots/rocky_planet.png"
    echo "   3. Presiona 2 → Toma captura → Guarda como: screenshots/gas_giant.png"
    echo "   4. Presiona 3 → Toma captura → Guarda como: screenshots/lava_planet.png"
    echo "   5. En macOS: Cmd+Shift+4 para captura de área"
    echo "   6. Vuelve a ejecutar este script después de tomar las capturas"
    echo ""
    
    read -p "¿Quieres ejecutar el programa ahora para tomar capturas? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cargo run --release
        echo ""
        echo "⚠️  Por favor, ejecuta este script de nuevo después de guardar las capturas"
        exit 0
    fi
else
    echo "✅ Todas las capturas presentes"
fi

echo ""

# Inicializar Git si no existe
if [ ! -d ".git" ]; then
    echo "🔧 Inicializando repositorio Git..."
    git init
    echo "✅ Repositorio Git inicializado"
else
    echo "✅ Repositorio Git ya existe"
fi

# Crear .gitignore si no existe
if [ ! -f ".gitignore" ]; then
    echo "📝 Creando .gitignore..."
    cat > .gitignore << 'EOF'
# Rust
/target/
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db
EOF
    echo "✅ .gitignore creado"
fi

# Verificar archivos importantes
echo ""
echo "📋 Verificando archivos del proyecto..."

files=(
    "Cargo.toml"
    "README.md"
    "INSTRUCCIONES.md"
    "NOTAS_TECNICAS.md"
    "RESUMEN.md"
    "src/main.rs"
    "src/obj.rs"
    "src/shader.rs"
    "assets/sphere.obj"
)

all_files_present=true
for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file"
    else
        echo "❌ $file (falta)"
        all_files_present=false
    fi
done

echo ""

if [ "$all_files_present" = false ]; then
    echo "⚠️  Algunos archivos importantes faltan"
    echo "   Revisa la lista anterior"
    exit 1
fi

# Git status
echo "📊 Estado del repositorio:"
git status --short

echo ""

# Preparar commit
if [ "$missing_screenshots" = false ]; then
    echo "🎯 Todo listo para hacer commit!"
    echo ""
    read -p "¿Quieres hacer commit ahora? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git add .
        git commit -m "feat: Planetary Shader Lab - 3 planetas procedurales

- Planeta Rocoso: superficie con cráteres y elevaciones
- Gigante Gaseoso: bandas atmosféricas y tormentas
- Planeta de Lava: flujo animado con emisión de luz

Técnicas implementadas:
- Ruido procedural 3D (Value Noise)
- Fractal Brownian Motion (FBM)
- Turbulencia
- Iluminación direccional
- Emisión de luz
- Animación temporal

Controles: 1,2,3 para cambiar planeta, flechas para cámara, +/- zoom"
        
        echo ""
        echo "✅ Commit realizado"
        echo ""
        echo "📤 Próximo paso: Crear repositorio en GitHub y hacer push"
        echo ""
        echo "Comandos sugeridos:"
        echo "  1. Ve a github.com y crea un nuevo repositorio 'planet_shaders'"
        echo "  2. Ejecuta:"
        echo "     git remote add origin https://github.com/TU_USUARIO/planet_shaders.git"
        echo "     git branch -M main"
        echo "     git push -u origin main"
    fi
else
    echo "⚠️  Esperando capturas de pantalla antes de hacer commit"
fi

echo ""
echo "🎉 Script completado!"
echo ""
echo "📝 Checklist final:"
echo "   ✅ Proyecto compilado"
echo "   $([ "$missing_screenshots" = false ] && echo "✅" || echo "⚠️ ") Capturas de pantalla"
echo "   ✅ Git inicializado"
echo "   $(git log --oneline 2>/dev/null | wc -l | tr -d ' ' | grep -q '^0$' && echo "⏳" || echo "✅") Commit realizado"
echo "   ⏳ Push a GitHub (hazlo manualmente)"
echo ""
