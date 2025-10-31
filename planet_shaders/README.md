# Planet Shaders - Laboratorio de Planetas Procedurales

[![Video Demo](https://img.youtube.com/vi/E7

Este proyecto implementa un renderizador 3D que genera planetas procedurales utilizando únicamente **shaders**, sin texturas ni materiales precargados. Todos los efectos visuales se crean mediante cálculos matemáticos y técnicas de ruido procedural en el fragment shader.

## Características Principales

### ✨ Tres Tipos de Planetas Implementados

1. **Planeta Rocoso** 🪨
   - Superficie rocosa con variaciones de color (tonos tierra/marrón)
   - Cráteres generados con ruido turbulento
   - Variaciones de elevación (montañas y valles)
   - Iluminación direccional realista

2. **Gigante Gaseoso** 🌪️
   - Bandas horizontales tipo Júpiter/Saturno
   - Turbulencia y remolinos atmosféricos
   - Gran tormenta circular animada (tipo Gran Mancha Roja)
   - Flujo de gases con animación temporal

3. **Planeta de Lava** 🌋 (Custom - Ciencia Ficción)
   - Superficie de lava fundida con flujo animado
   - Grietas brillantes con emisión de luz propia
   - Pulsación de calor sincronizada
   - Zonas de roca enfriada y lava activa

### 🎨 Técnicas de Shader Implementadas

- **Ruido Procedural 3D**: Generación de patrones pseudo-aleatorios
- **Fractal Brownian Motion (FBM)**: Múltiples octavas de ruido para detalles complejos
- **Turbulencia**: Ruido absoluto para efectos de cráteres y grietas
- **Interpolación de Colores**: Mezclas suaves entre múltiples paletas
- **Iluminación Direccional**: Cálculo de intensidad lumínica basada en normales
- **Emisión de Luz**: Simulación de materiales auto-luminosos (lava)
- **Animación Temporal**: Efectos dinámicos basados en el tiempo

### 🎮 Controles

| Tecla | Acción |
|-------|--------|
| **1** | Planeta Rocoso |
| **2** | Gigante Gaseoso |
| **3** | Planeta de Lava |
| **←/→** | Rotar cámara horizontalmente |
| **↑/↓** | Rotar cámara verticalmente |
| **+/-** | Zoom in/out |
| **ESPACIO** | Activar/desactivar rotación automática del planeta |
| **ESC** | Salir |

## Instalación y Ejecución

### Requisitos

- **Rust** (versión 1.70 o superior)
- **Cargo** (incluido con Rust)
- Sistema operativo: Windows, macOS o Linux

### Pasos de Instalación

1. **Clonar el repositorio**:
```bash
git clone https://github.com/tu-usuario/planet_shaders.git
cd planet_shaders
```

2. **Asegurarse de que `sphere.obj` está en la carpeta `assets/`**:
```bash
# El archivo debe estar en: assets/sphere.obj
# Si no está, copiarlo desde static_shaders/
cp ../static_shaders/sphere.obj assets/
```

3. **Compilar y ejecutar**:
```bash
cargo run --release
```

El flag `--release` es recomendado para mejor rendimiento (60 FPS).

## Estructura del Proyecto

```
planet_shaders/
├── Cargo.toml          # Configuración de dependencias
├── README.md           # Este archivo
├── assets/
│   └── sphere.obj      # Modelo 3D de esfera base
└── src/
    ├── main.rs         # Renderizador principal y loop de juego
    ├── obj.rs          # Cargador de archivos .obj
    └── shader.rs       # Implementación de todos los shaders
```

## Detalles Técnicos

### Arquitectura del Shader

El sistema de shaders está diseñado de forma modular:

```rust
pub enum PlanetType {
    Rocky,      // Planeta rocoso
    GasGiant,   // Gigante gaseoso
    Lava,       // Planeta de lava
}

pub struct Fragment {
    pub world_position: Vector3,  // Posición en espacio mundial
    pub normal: Vector3,           // Normal de la superficie
}

pub struct Uniforms {
    pub time: f32,                 // Tiempo de animación
    pub planet_type: PlanetType,   // Tipo de planeta activo
}
```

### Funciones de Ruido

1. **hash(p: Vector3) -> f32**: Genera valores pseudo-aleatorios consistentes
2. **noise3d(p: Vector3) -> f32**: Ruido de valor 3D con interpolación cúbica
3. **fbm(p: Vector3, octaves: i32) -> f32**: Fractal Brownian Motion
4. **turbulence(p: Vector3, octaves: i32) -> f32**: Ruido turbulento

### Pipeline de Renderizado

1. **Carga del Modelo**: `tobj` parsea `sphere.obj` → vértices e índices
2. **Cálculo de Normales**: Normales por triángulo basadas en geometría esférica
3. **Transformaciones**: Rotación del planeta + transformación de cámara
4. **Backface Culling**: Elimina caras no visibles
5. **Proyección Perspectiva**: Espacio 3D → pantalla 2D
6. **Fragment Shader**: Calcula color procedural por triángulo
7. **Rasterización**: `draw_triangle()` de raylib

## Capturas de Pantalla

### Planeta Rocoso
![Planeta Rocoso](screenshots/rocky_planet.png)
*Superficie con cráteres y variaciones de elevación*

### Gigante Gaseoso
![Gigante Gaseoso](screenshots/gas_giant.png)
*Bandas atmosféricas y gran tormenta roja*

### Planeta de Lava
![Planeta de Lava](screenshots/lava_planet.png)
*Superficie fundida con emisión de luz*

## Criterios de Evaluación Cumplidos

- ✅ **Tres planetas distintos**: Rocoso, Gaseoso, Lava
- ✅ **Sin texturas**: Todo generado proceduralmente
- ✅ **Modelo base único**: Todos usan la misma esfera
- ✅ **Shaders avanzados**: FBM, turbulencia, iluminación
- ✅ **Animación**: Efectos temporales en todos los planetas
- ✅ **Controles interactivos**: Cambio entre planetas en tiempo real
- ✅ **Rendimiento**: 60 FPS en modo release
- ✅ **Documentación**: README completo con instrucciones

## Tecnologías Utilizadas

- **Rust** - Lenguaje de programación
- **raylib** - Biblioteca de gráficos 2D/3D
- **tobj** - Parser de archivos .obj
- **Matemáticas procedurales** - Ruido Perlin/Value, FBM

## Autor

Ernesto Ascencio
Universidad del Valle de Guatemala
Fecha: Octubre 2025

## Licencia

MIT License - Ver archivo LICENSE para más detalles
