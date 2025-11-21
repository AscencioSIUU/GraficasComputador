# Solar System - Software Renderer

> **Video de Demostración:**  
> *[Insertar enlace al video aquí]*

---

Simulación interactiva del sistema solar con renderer por software, shaders procedurales y vertex displacement.

## Estructura del Proyecto

### Archivos Raíz

- **`Cargo.toml`** - Configuración del proyecto Rust con dependencias (raylib, noise, tobj)
- **`README.md`** - Este archivo de documentación
- **`.gitignore`** - Archivos ignorados por Git (target/, binarios)
- **`sphere.obj`** - Modelo 3D de esfera base (densidad media) usado para planetas
- **`sphere_2000.obj`** - Modelo 3D de esfera alta densidad (2048 triángulos) para mayor detalle
- **`spaceship.obj`** - Modelo 3D de la nave espacial del jugador

### Directorio `src/`

#### Archivos Principales

- **`main.rs`** - Punto de entrada principal del programa. Contiene:
  - Loop principal de renderizado
  - Sistema de cámara (libre y tercera persona)
  - Física orbital de los planetas
  - Renderizado del skybox (estrellas y nebulosa animada)
  - Sistema de cometas aleatorios
  - Efecto de warp instantáneo
  - Renderización de órbitas planetarias
  - Sistema de glow para planetas especiales (Aeon)
  - Renderización de la luna de Kleos con órbita inclinada
  - Depth sorting para correcta visualización 3D
  - UI de controles y panel de teletransporte

- **`obj.rs`** - Cargador de archivos OBJ en formato Wavefront. Parsea:
  - Vértices (v)
  - Normales (vn)
  - Coordenadas de textura (vt)
  - Caras (f)
  - Convierte la geometría en arrays de `Vector3` para renderizado

- **`controls.rs`** - Sistema de control de la nave espacial. Maneja:
  - Input de teclado (WASD, flechas)
  - Cálculo de vectores de dirección (forward, right, up)
  - Rotación de la nave (pitch, yaw, roll)
  - Transformación de coordenadas locales a globales

- **`shader.rs`** - Archivo legacy (deprecado, puede estar vacío)

- **`ship.rs`** - Archivo legacy relacionado con la nave (deprecado)

#### Directorio `src/shaders/`

Contiene todos los shaders procedurales del sistema solar:

- **`mod.rs`** - Módulo raíz de shaders. Exporta:
  - Todos los shaders de planetas
  - Funciones de vertex displacement
  - Estructuras compartidas (Fragment, Uniforms)
  - Traits y utilidades

- **`common.rs`** - Funciones base de ruido procedural compartidas:
  - `perlin_noise_3d()` - Ruido Perlin suave y continuo
  - `simplex_noise_3d()` - Ruido Simplex (mejor que Perlin)
  - `voronoi_noise_3d()` - Patrón de celdas Voronoi
  - `cellular_noise_3d()` - Ruido celular tipo Worley
  - `fbm_noise_3d()` - Fractional Brownian Motion (múltiples octavas)
  - `fbm()` - Versión sobrecargada de FBM
  - Estructura `Fragment` - Datos de geometría (posición, normal)
  - Estructura `Uniforms` - Parámetros uniformes (tiempo, intensidad)
  - Trait `Vector3Ext` - Extensiones para Vector3 (lerp, dot, normalized)

- **`advanced_noise.rs`** - Funciones avanzadas de ruido procedural:
  - `worley_noise_3d()` - Ruido celular para atmósferas suaves
  - `value_noise_3d()` - Ruido de interpolación simple y rápido
  - `domain_warp_3d()` - Distorsión del espacio antes de aplicar noise
  - `ridged_multifractal()` - Crestas invertidas estilo montañas
  - `fbm_enhanced()` - FBM configurable (lacunarity, gain, octaves)

- **`sun.rs`** - Shader del Sol (índice 0):
  - `vertex_displacement()` - Corona solar masiva con prominencias
  - `star_shader()` - Emisión solar intensa con múltiples capas de ruido
  - Colores: Rojo nuclear, naranja, amarillo
  - Efectos: Pulsación, manchas solares, corona radiante

- **`mercury.rs`** - Shader de Aeon (índice 1, antes Mercury):
  - `vertex_displacement_mercury()` - Cráteres y exosfera con domain warping
  - `cellular_planet_shader()` - Superficie rocosa con cellular noise
  - Colores: Azul marino oscuro con corona azul brillante
  - Efectos: Rim light azul para atmósfera

- **`venus.rs`** - Shader de Thalassa (índice 2, antes Venus):
  - `vertex_displacement_venus()` - Atmósfera densa con ridged multifractal
  - `simplex_planet_shader()` - Nubes ácidas turbulentas
  - Colores: Amarillo-beige
  - Efectos: Nubes animadas, turbulencia atmosférica

- **`earth.rs`** - Shader legacy de Earth (no usado actualmente):
  - `vertex_displacement_earth()` - Terreno oceánico y montañoso con Worley
  - `voronoi_planet_shader()` - Océanos y continentes con Voronoi
  - *Nota: Kleos usa su propio shader especializado*

- **`kleos.rs`** - Shader de Kleos/Tierra (índice 3):
  - `kleos_shader()` - Superficie lisa estilo Saturno con bandas horizontales
  - Colores: Gradiente verde-azul (océanos y tierra)
  - Efectos: Nubes suaves, rim light azul cielo, bandas de latitud

- **`mars.rs`** - Shader de Kefi (índice 4, antes Mars):
  - `vertex_displacement_mars()` - Cañones profundos y tormentas de polvo
  - `perlin_planet_shader()` - Desierto rojizo con Perlin noise
  - Colores: Rojo-naranja marciano
  - Efectos: Terreno árido, variaciones de altura

- **`saturn.rs`** - Shader de Agape (índice 5, antes Saturn):
  - `vertex_displacement_saturn()` - Atmósfera suave y sutil
  - `saturn_shader()` - Bandas horizontales amarillo-beige
  - Colores: Paleta de 5 beiges (amarillo pálido a marrón claro)
  - Efectos: Bandas latitudinales con transiciones suaves

- **`goliath.rs`** - Shader de Goliath (índice 6):
  - `vertex_displacement_goliath()` - Tormentas de gas masivas con value noise
  - `planet_shader()` - Bandas moradas con toques negros
  - Colores: Negro profundo a magenta neón brillante
  - Efectos: Aura expandida morada, rim light ultra intenso, bandas negras

- **`spaceship.rs`** - Shader de la nave espacial:
  - `spaceship_shader()` - Iluminación dinámica metálica
  - Efectos: Reflejos especulares, sombreado realista

### Directorio `target/`

Directorio de compilación generado por Cargo (no versionado en Git):

- **`debug/`** - Binarios y artefactos de compilación en modo debug
- **`release/`** - Binarios y artefactos de compilación en modo release (optimizado)
- **`.rustc_info.json`** - Información de caché del compilador Rust
- **`CACHEDIR.TAG`** - Marcador de directorio de caché

---

## Configuración de Planetas

| Índice | Nombre | Órbita | Tamaño | Color Base | Shader | Anillos | Luna |
|--------|--------|--------|--------|------------|--------|---------|------|
| 0 | **Sol** | 0 | 18.0 | Amarillo | `star_shader` | ❌ | ❌ |
| 1 | **Aeon** | 100 | 4.5 | Azul marino oscuro | `cellular_planet_shader` | ✅ Neón azul | ❌ |
| 2 | **Thalassa** | 55 | 5.4 | Amarillo-beige | `simplex_planet_shader` | ❌ | ❌ |
| 3 | **Kleos** | 75 | 6.0 | Verde-azul | `kleos_shader` | ❌ | ✅ 45° inclinada |
| 4 | **Kefi** | 35 | 3.6 | Rojo-naranja | `perlin_planet_shader` | ❌ | ❌ |
| 5 | **Agape** | 130 | 11.0 | Beige-amarillo | `saturn_shader` | ✅ Tradicionales | ❌ |
| 6 | **Goliath** | 165 | 13.2 | Negro-morado neón | `planet_shader` | ❌ | ❌ |

---

## Efectos Visuales Especiales

### Nebulosa de Fondo
- Triple capa de Perlin noise con colores rojo, azul y púrpura
- Animación temporal suave
- Pixel step de 4 para optimización

### Cometas Aleatorios
- Aparición cada 1-4 segundos
- Órbitas desde borde del skybox (radio 480)
- Velocidad: 120-180 unidades/segundo
- Duración variable: 8-12 segundos
- Cola renderizada con 25 segmentos conectados
- Fade in/out suave

### Efectos Planetarios
- **Aeon**: Glow azul de 6 capas + anillos neón pulsantes azules + órbita azul
- **Goliath**: Aura morada expandida ultra brillante con negros profundos
- **Kleos**: Luna con órbita inclinada 45° (15 veces el radio del planeta)

### Warp Instantáneo
- Activación inmediata (teclas 1-7)
- Efecto de líneas estelares desde el centro
- Animación de aceleración progresiva

---

## Instalación y Ejecución

### Prerrequisitos
- **Rust** 1.70 o superior ([Instalar desde rustup.rs](https://rustup.rs/))
- **Cargo** (incluido con Rust)
- **Git** para clonar el repositorio

### Clonar Repositorio
```bash
git clone https://github.com/AscencioSIUU/GraficasComputador.git
cd GraficasComputador/solar_system
```

### Compilar y Ejecutar
```bash
# Modo debug (compilación rápida, ejecución lenta)
cargo run

# Modo release (compilación lenta, ejecución optimizada) - RECOMENDADO
cargo run --release
```

### Compilar sin ejecutar
```bash
cargo build --release
```

El ejecutable se generará en `target/release/solar_system`

---

## Controles

### Movimiento de la Nave
| Tecla | Función |
|-------|---------|
| **W** | Adelante |
| **S** | Atrás |
| **A** | Izquierda |
| **D** | Derecha |
| **↑** | Rotar hacia arriba (pitch) |
| **↓** | Rotar hacia abajo (pitch) |
| **←** | Rotar izquierda (yaw) |
| **→** | Rotar derecha (yaw) |

⚠️ **Nota**: La nave solo se mueve en el plano X-Z (horizontal). No hay movimiento vertical directo.

### Cámara y Vistas
| Tecla | Función |
|-------|---------|
| **V** | Cambiar a vista elevada del sistema (cámara libre) |
| **T** | Pausar/Reanudar simulación (pausa órbitas y tiempo) |

### Teletransporte Instantáneo (Warp)
| Tecla | Destino |
|-------|---------|
| **1** | ☀️ Sol |
| **2** | 🪐 Aeon (planeta azul con anillos neón) |
| **3** | 🌕 Thalassa (planeta amarillo) |
| **4** | 🌍 Kleos (Tierra con luna) |
| **5** | 🔴 Kefi (planeta rojo) |
| **6** | 🪐 Agape (gigante con anillos) |
| **7** | 💜 Goliath (gigante morado neón) |

### Sistema
| Tecla | Función |
|-------|---------|
| **ESC** | Salir del programa |

---

## Tecnologías y Dependencias

### Librerías Rust (Cargo.toml)

- **[raylib](https://crates.io/crates/raylib)** `5.5+` - Framework gráfico multiplataforma
  - Ventana y contexto de renderizado
  - Funciones de dibujo 2D (primitivas, líneas, círculos)
  - Manejo de input (teclado, mouse)
  - Control de tiempo y FPS

- **[noise](https://crates.io/crates/noise)** `0.9` - Generación de ruido procedural
  - Perlin noise 3D
  - Simplex noise 3D
  - Funciones base de noise para shaders

- **[tobj](https://crates.io/crates/tobj)** `4.0` - Cargador de archivos OBJ/MTL
  - Parseo de geometría Wavefront OBJ
  - Soporte para vértices, normales y UVs
  - Manejo de múltiples meshes

### Características Técnicas

- **Renderer**: Software rasterizer (sin GPU shader pipeline)
- **Proyección**: Perspectiva con FOV de 70°
- **Culling**: Backface culling para optimización
- **Sorting**: Depth sorting (painter's algorithm) para correcta superposición
- **Shaders**: Procedurales por triángulo (no per-pixel)
- **Física**: Órbitas Keplerianas simplificadas
- **Optimización**: Early rejection, minimal noise calls, batched rendering

---

## Especificaciones de Rendimiento

### Complejidad Geométrica
- **Planetas**: ~960 triángulos cada uno (sphere.obj)
- **Luna**: ~960 triángulos
- **Nave**: Variable según modelo
- **Total**: ~8000-10000 triángulos por frame

### Optimizaciones Implementadas
1. **Backface Culling**: Descarta ~50% de triángulos
2. **Early Rejection**: Evita proyección de triángulos fuera de vista
3. **Shader per-triangle**: En vez de per-pixel (reduce de millones a ~1500 llamadas)
4. **Depth Sorting**: Una sola vez por frame
5. **Noise Caching**: Reducción de octavas en FBM

---

## Características Destacadas

### Sistema de Shaders Procedurales
- Cada planeta tiene shader único con noise específico
- Vertex displacement para geometría dinámica
- Rim lighting para atmósferas
- Corona y auras personalizadas

### Efectos Visuales
- ✨ Nebulosa animada de fondo (triple Perlin noise)
- ☄️ Cometas aleatorios con colas de 25 segmentos
- 🌟 Skybox de 2000 estrellas fijas
- 🌀 Efecto warp con líneas estelares
- 💫 Glow y auras para planetas especiales

### Sistema de Anillos
- **Aeon**: 6 anillos giratorios neón azules con pulsación
- **Agape**: 60 anillos concéntricos tradicionales estáticos
- Renderizado con depth-aware occlusion

### Física Orbital
- Velocidades orbitales diferenciadas
- Rotación (spin) independiente por planeta
- Luna con órbita inclinada 45° realista

---

## Conceptos de Gráficos Implementados

### Renderizado 3D
- Transformaciones de modelo, vista y proyección
- Proyección perspectiva manual
- Rotación en 3 ejes (X, Y, Z)
- Cámara libre y tercera persona

### Iluminación
- Diffuse lighting (Lambert)
- Rim lighting (Fresnel aproximado)
- Emissive materials (Sol)
- Specular highlights (océanos de Kleos)

### Noise Procedural
- Perlin Noise (coherente y suave)
- Simplex Noise (más rápido que Perlin)
- Voronoi/Cellular (patrones de celdas)
- Worley Noise (atmósferas suaves)
- Domain Warping (distorsión espacial)
- Fractional Brownian Motion (múltiples octavas)

### Técnicas de Optimización
- Frustum culling
- Backface culling
- Z-buffer simulation (depth sorting)
- LOD conceptual (diferentes densidades de esfera)

---

## Requisitos del Sistema

### Mínimos
- **CPU**: Dual-core 2.0 GHz o superior
- **RAM**: 512 MB
- **Sistema Operativo**: Windows 10/11, macOS 10.15+, Linux (kernel 4.0+)
- **Espacio en disco**: 100 MB

### Recomendados
- **CPU**: Quad-core 3.0 GHz o superior
- **RAM**: 2 GB
- **Sistema Operativo**: Windows 11, macOS 13+, Linux reciente

---

## Solución de Problemas

### Error al compilar
```bash
# Actualizar Rust a la última versión
rustup update

# Limpiar caché y recompilar
cargo clean
cargo build --release
```

### FPS bajo
- Asegúrate de usar `--release` flag
- Cierra otras aplicaciones pesadas
- Considera reducir la densidad de la esfera (usar sphere.obj en vez de sphere_2000.obj)

### Modelos OBJ no cargan
- Verifica que `sphere.obj` y `spaceship.obj` estén en la raíz del proyecto
- Revisa los mensajes de consola para rutas intentadas

---

## Referencias

- [Raylib Documentation](https://www.raylib.com/)
- [Perlin Noise Explanation](https://adrianb.io/2014/08/09/perlinnoise.html)
- [Software Rendering Fundamentals](https://www.scratchapixel.com/)
- [Procedural Planets Tutorial](https://www.youtube.com/watch?v=QN39W020LqU)

--- 
# Autor 
Ernesto David Ascencio Ramírez - 23009
**erneram** - [GitHub](https://github.com/erneram)
Universidad del Valle de Guatemala


