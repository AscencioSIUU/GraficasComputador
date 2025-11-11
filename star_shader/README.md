# 🌟 Star Shader Lab - Animated Sun with Perlin Noise

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## 🔗 Repositorio de GitHub

**Repositorio funcional:** [https://github.com/AscencioSIUU/GraficasComputador](https://github.com/AscencioSIUU/GraficasComputador)

Este proyecto está completamente funcional y listo para clonar y ejecutar. Incluye todos los archivos fuente, assets, documentación y ejemplos visuales.

---

## 📝 Descripción del Proyecto

Este proyecto implementa una **estrella animada (sol)** utilizando exclusivamente **shaders procedurales** y **funciones de ruido Perlin**. La estrella simula actividad solar realista incluyendo turbulencia, manchas solares, erupciones, y pulsaciones, todo generado en tiempo real mediante técnicas de ruido procedural.

### 🎬 Animación en Tiempo Real

![Star Animation](assets/video.gif)

*GIF mostrando la animación continua de la estrella con turbulencia extrema, erupciones solares, manchas oscuras y pulsaciones de corona.*

## ✨ Características Principales

### 🎨 Tipos de Ruido Utilizados y su Impacto

Este proyecto utiliza tres tipos de ruido procedural de la librería `noise` crate de Rust. Cada tipo de ruido tiene características únicas que afectan diferentes aspectos visuales de la estrella:

| Tipo de Ruido | Librería | Rango de Valores | Uso en el Shader | Impacto en Color/Intensidad |
|--------------|----------|------------------|------------------|------------------------------|
| **Perlin Noise 3D** | `noise::Perlin` | [-1.0, 1.0] → Normalizado a [0, 1] | Manchas solares, corona base | **Intensidad**: Crea variaciones suaves en el brillo. Valores bajos (<0.3) generan manchas oscuras. **Color**: No afecta directamente, pero reduce la intensidad localmente. |
| **Fractal Brownian Motion (FBM)** | `noise::Fbm<Perlin>` | [-1.0, 1.0] → Normalizado a [0, 1] | Turbulencia base (6-8 octavas), granulación superficial (4-6 octavas) | **Intensidad**: Suma múltiples escalas de detalle (0.5-1.5 range). Aumenta la complejidad visual con picos y valles. **Color**: La intensidad resultante se mapea al gradiente de temperatura (rojo→amarillo→azul). |
| **Turbulence** | `noise::Turbulence<Perlin>` | [0.0, ~1.5] (valores absolutos) | Erupciones solares, flares | **Intensidad**: Genera picos agresivos de energía (hasta 2.5). Valores >1.0 crean "erupciones" brillantes. **Color**: Intensidades altas (>1.2) se vuelven casi blancas (emisión extrema). |

### 🌈 Cómo el Ruido Afecta el Color y la Intensidad

El flujo de transformación es el siguiente:

```
Ruido Procedural → Intensidad Combinada → Mapeo de Temperatura → Color Final + Emisión
```

#### 1️⃣ **Generación de Intensidad Base**
```rust
// FBM para turbulencia (valores ~0.3 - 1.2)
let turbulence = fbm_noise_3d(pos.x * 8.0, pos.y * 8.0, pos.z * 8.0, 8);
let turbulence_normalized = (turbulence + 1.0) * 0.5; // [-1,1] → [0,1]

// Turbulence para erupciones (valores 0.0 - 1.5)
let flares = turbulence_noise_3d(pos.x * 12.0, pos.y * 12.0 + time, pos.z * 12.0);
let flares_normalized = (flares + 1.0) * 0.5;

// Perlin para manchas solares (valores -0.8 a 0.0)
let sunspots = perlin_noise_3d(pos.x * 4.0, pos.y * 4.0, pos.z * 4.0);
let sunspot_factor = if sunspots < 0.0 { sunspots.abs() * 0.8 } else { 0.0 };
```

#### 2️⃣ **Combinación de Intensidades**
```rust
// Cada tipo de ruido contribuye de forma diferente
let mut intensity = 0.8;                           // Base (brillante)
intensity += turbulence_normalized.powf(0.6) * 0.6; // Turbulencia (+20-60%)
intensity -= sunspot_factor * 0.7;                  // Manchas oscuras (-50%)
intensity += flares_normalized * 4.0;               // Erupciones (+400%!)
intensity = intensity.clamp(0.1, 2.5);              // Límites finales
```

**Resultado**: 
- Zonas tranquilas: intensidad ~0.7-1.0 (brillo normal)
- Manchas solares: intensidad ~0.2-0.4 (oscuro)
- Erupciones activas: intensidad 1.5-2.5 (super brillante)

#### 3️⃣ **Mapeo de Intensidad a Color (Temperatura)**
```rust
fn temperature_to_color(intensity: f32, temp_factor: f32) -> Vector3 {
    let base_color = if temp_factor < 0.3 {
        // Estrella ROJA (fría): Betelgeuse
        Vector3::new(0.9, 0.2, 0.05)  // RGB dominante en rojo
    } else if temp_factor < 0.6 {
        // Sol AMARILLO (medio): Nuestro Sol
        Vector3::new(0.95, 0.8, 0.3)  // RGB equilibrado, amarillo
    } else {
        // Estrella AZUL (caliente): Rigel
        Vector3::new(0.7, 0.85, 0.95) // RGB dominante en azul
    };
    
    // La intensidad modula el brillo
    base_color * intensity
}
```

**Impacto del Ruido en Color**:
- **Turbulencia (FBM)**: Crea variaciones sutiles en brillo, haciendo que el amarillo/rojo/azul sea más o menos intenso.
- **Erupciones (Turbulence)**: En intensidades >1.5, los colores se saturan hacia blanco (efecto de "incandescencia").
- **Manchas (Perlin)**: Reducen intensidad localmente, oscureciendo el color sin cambiar el tono.

#### 4️⃣ **Emisión de Luz Variable**
```rust
// Emisión cuadrática: cuanto más intenso, más "brilla"
let emission = (intensity * intensity) * 2.0;  // Rango: 0.02 - 12.5

// Color final emitido
let emissive_color = base_color * Vector3::new(
    0.8 * emission,  // Canal rojo
    0.7 * emission,  // Canal verde
    0.6 * emission   // Canal azul
);
```

**Efecto Visual**:
- Zonas normales (intensidad ~1.0): Emisión ~2.0 (brillo estándar)
- Erupciones (intensidad ~2.0): Emisión ~8.0 (4x más brillante, casi blanco)
- Manchas oscuras (intensidad ~0.3): Emisión ~0.18 (muy tenue)

---

### 📊 Resumen: Ruido → Color/Intensidad

| Efecto | Tipo de Ruido | Contribución a Intensidad | Impacto en Color | Rango Visual |
|--------|---------------|---------------------------|------------------|--------------|
| **Turbulencia Base** | FBM (8 octavas) | +0.0 a +0.6 | Modula brillo del color base | Variaciones suaves |
| **Manchas Solares** | Perlin 3D | -0.0 a -0.7 | Oscurece color significativamente | Zonas oscuras |
| **Erupciones** | Turbulence | +0.0 a +4.0 | Satura hacia blanco en picos | Flashes brillantes |
| **Granulación** | FBM (6 octavas) | ±0.2 (detalle fino) | Textura superficial sutil | Células pequeñas |
| **Corona (Vertex)** | Perlin + FBM | N/A (geometría) | Cambia forma, no color | Protuberancias |

**Conclusión**: El ruido procedural controla completamente la apariencia visual. Sin ruido, la estrella sería una esfera uniforme de un solo color. Con ruido, obtenemos una superficie viva y dinámica con manchas oscuras, regiones brillantes, y erupciones violentas, todo animado en tiempo real.

### 🎮 Controles Interactivos

| Tecla | Acción |
|-------|--------|
| **← →** | Rotar cámara horizontalmente |
| **↑ ↓** | Rotar cámara verticalmente |
| **ESPACIO** | Toggle rotación automática |
| **I / K** | Aumentar/Disminuir intensidad |
| **T / G** | Aumentar/Disminuir temperatura |
| **V** | Toggle vertex displacement (corona) |
| **1** | Preset: Sol amarillo (tipo G) |
| **2** | Preset: Gigante roja (tipo M - Betelgeuse) |
| **3** | Preset: Estrella azul (tipo B - Rigel) |
| **ESC** | Salir |

---

## 📦 Instalación y Ejecución

### Requisitos

- **Rust** 1.70+
- **Cargo**
- Sistema operativo: Windows, macOS o Linux

### Dependencias

```toml
[dependencies]
raylib = "5.5"      # Renderizado gráfico
tobj = "4.0"        # Carga de archivos .obj
noise = "0.9"       # Funciones de ruido Perlin
```

### Pasos de Instalación

1. **Clonar el repositorio desde GitHub**:
```bash
git clone https://github.com/AscencioSIUU/GraficasComputador.git
cd GraficasComputador/star_shader
```

2. **Colocar `sphere.obj` en `assets/`**:
```bash
# Si ya tienes sphere.obj de planet_shaders:
cp ../planet_shaders/assets/sphere.obj assets/

# O descárgalo de cualquier fuente de modelos 3D
# El código busca automáticamente en múltiples rutas
```

3. **Compilar y ejecutar**:
```bash
cargo run --release
```

El flag `--release` es **altamente recomendado** para 60 FPS estables.

**Vista Inicial**: Plano horizontal para apreciar mejor el movimiento del ruido extremo y las erupciones solares.

---

## 🔬 Detalles Técnicos

### Arquitectura del Shader

El shader está dividido en módulos funcionales:

```rust
pub struct Uniforms {
    pub time: f32,        // Tiempo de animación
    pub intensity: f32,   // Intensidad global (0.1 - 2.0)
    pub temperature: f32, // Temperatura (0.0 = roja, 1.0 = azul)
}

pub fn star_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3
pub fn vertex_displacement(position: Vector3, time: f32) -> Vector3
```

### Flujo del Shader

```
1. Perlin Noise 3D
   ↓
2. FBM (múltiples octavas) → Turbulencia base
   ↓
3. Turbulence → Erupciones solares
   ↓
4. Threshold filtering → Manchas solares
   ↓
5. Combinación de intensidades
   ↓
6. Mapeo intensidad → color (temperatura)
   ↓
7. Emisión de luz variable
   ↓
8. Color final RGB
```

### Explicación de Funciones de Ruido

A continuación se documenta cada función de ruido utilizada en el shader, incluyendo sus parámetros, comportamiento y uso específico.

---

#### 1. **Perlin Noise 3D** - Ruido Suave y Continuo

```rust
fn perlin_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    let perlin = Perlin::new(42);  // Seed fija para reproducibilidad
    perlin.get([x, y, z]) as f32
}
```

**Documentación de la Función:**
- **Parámetros de entrada**: 
  - `x`, `y`, `z`: Coordenadas 3D en espacio de ruido (tipo `f64`)
- **Retorno**: Valor de ruido en rango `[-1.0, 1.0]` (tipo `f32`)
- **Seed**: `42` (constante) - Garantiza que el ruido sea idéntico entre ejecuciones
- **Uso en shader**: Manchas solares, variaciones base de corona
- **Características**: 
  - Gradientes suaves sin discontinuidades
  - Coherente espacialmente (valores cercanos son similares)
  - Determinístico (misma entrada = misma salida)

**Aplicación en Star Shader:**
```rust
// Manchas solares (zonas oscuras)
let sunspot_noise = perlin_noise_3d(
    pos.x as f64 * 4.0 + time as f64 * 0.2,  // Escala espacial 4x, movimiento lento
    pos.y as f64 * 4.0 + time as f64 * 0.2,
    pos.z as f64 * 4.0
);

// Normalización: [-1, 1] → [0, 1]
let sunspot_normalized = (sunspot_noise + 1.0) * 0.5;

// Threshold: solo valores < 0.0 son manchas oscuras
let sunspot_darkness = if sunspot_noise < 0.0 { 
    sunspot_noise.abs() * 0.8  // Máximo 80% de oscuridad
} else { 
    0.0 
};
```

---

#### 2. **Fractal Brownian Motion (FBM)** - Ruido Multi-Escala

```rust
fn fbm_noise_3d(x: f64, y: f64, z: f64, octaves: usize) -> f32 {
    let fbm = Fbm::<Perlin>::new(42).set_octaves(octaves);
    fbm.get([x, y, z]) as f32
}
```

**Documentación de la Función:**
- **Parámetros de entrada**: 
  - `x`, `y`, `z`: Coordenadas 3D en espacio de ruido
  - `octaves`: Número de capas de detalle (típicamente 4-8)
- **Retorno**: Valor de ruido combinado en rango `[-1.0, 1.0]` aproximado
- **Algoritmo**: Suma múltiples octavas de Perlin con frecuencias crecientes
  - Octava 1: Frecuencia base (detalles grandes)
  - Octava 2: Frecuencia 2x (detalles medianos)
  - Octava 3: Frecuencia 4x (detalles pequeños)
  - ... hasta `octaves` capas
- **Uso en shader**: Turbulencia compleja, granulación superficial

**Aplicación en Star Shader:**
```rust
// Turbulencia solar (8 octavas para máximo detalle)
let turbulence = fbm_noise_3d(
    pos.x as f64 * 8.0 + time as f64 * 0.4,  // Escala 8x, rápido
    pos.y as f64 * 8.0 + time as f64 * 0.5,
    pos.z as f64 * 8.0 + time as f64 * 0.4,
    8  // 8 octavas = detalles desde grandes remolinos hasta células finas
);

// Normalización y ajuste de contraste
let turbulence_normalized = (turbulence + 1.0) * 0.5;
let turbulence_enhanced = turbulence_normalized.powf(0.6);  // Aumenta contraste

// Granulación superficial (6 octavas, alta frecuencia)
let granulation = fbm_noise_3d(
    pos.x as f64 * 30.0 + time as f64 * 1.5,  // Escala 30x (células pequeñas)
    pos.y as f64 * 30.0 + time as f64 * 1.8,
    pos.z as f64 * 30.0 + time as f64 * 1.5,
    6  // 6 octavas
);
let granulation_normalized = (granulation + 1.0) * 0.5;
```

**Diferencia entre octavas**:
- **4 octavas**: Detalles gruesos, rápido de calcular
- **6 octavas**: Balance entre detalle y performance
- **8 octavas**: Máximo detalle, textura muy compleja (usado en turbulencia principal)

---

#### 3. **Turbulence** - Ruido Caótico Absoluto

```rust
fn turbulence_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    let turbulence = Turbulence::<_, Perlin>::new(Perlin::new(42));
    turbulence.get([x, y, z]) as f32
}
```

**Documentación de la Función:**
- **Parámetros de entrada**: 
  - `x`, `y`, `z`: Coordenadas 3D en espacio de ruido
- **Retorno**: Valor de ruido **siempre positivo** en rango `[0.0, ~1.5]`
- **Algoritmo**: Calcula `abs(perlin_noise)` en cada octava, luego suma
  - Resultado: Patrones más agresivos y "filosos"
  - No hay valores negativos (todas las variaciones son "picos")
- **Uso en shader**: Erupciones violentas, flares energéticos

**Aplicación en Star Shader:**
```rust
// Erupciones solares (flares)
let flares = turbulence_noise_3d(
    pos.x as f64 * 12.0 + time as f64 * 1.2,  // Alta frecuencia, muy rápido
    pos.y as f64 * 12.0 + time as f64 * 1.2,
    pos.z as f64 * 12.0
);

// Normalización conservadora (Turbulence puede exceder 1.0)
let flares_normalized = (flares + 1.0) * 0.5;
let flares_clamped = flares_normalized.clamp(0.0, 1.0);

// Pulsación periódica (ciclo solar)
let pulse = (time * 3.0).sin() * 0.5 + 0.5;  // [0, 1] onda sinusoidal
let flare_intensity = flares_clamped * pulse * 4.0;  // Picos hasta 4x!
```

**Por qué Turbulence es ideal para erupciones**:
- **Valores absolutos**: Crea "explosiones" en lugar de variaciones suaves
- **Patrones agresivos**: Bordes más definidos, contraste extremo
- **Siempre positivo**: Perfecto para "añadir energía" sin crear zonas negativas

---

### 📋 Documentación de Uniforms (Variables Globales del Shader)

```rust
pub struct Uniforms {
    pub time: f32,        // Tiempo de animación (segundos desde inicio)
    pub intensity: f32,   // Intensidad global de la estrella [0.1 - 2.0]
    pub temperature: f32, // Factor de temperatura [0.0 - 1.0]
}
```

**Documentación Detallada:**

| Uniform | Tipo | Rango | Descripción | Control | Efecto Visual |
|---------|------|-------|-------------|---------|---------------|
| **`time`** | `f32` | `[0.0, ∞)` | Tiempo transcurrido en segundos desde el inicio del programa. Se incrementa continuamente en cada frame. | Automático (no controlable) | **Animación continua**: Todos los efectos de ruido se animan sumando `time` a sus coordenadas. Ciclos periódicos con `sin(time)` y `cos(time)`. |
| **`intensity`** | `f32` | `[0.1, 2.0]` | Multiplicador global de brillo. Valor base: `1.0` (normal). `< 1.0`: estrella apagándose. `> 1.0`: estrella más activa. | Teclas `I` (incrementar) / `K` (decrementar) | **Brillo global**: Multiplica la intensidad calculada por ruido. Afecta emisión de luz cuadráticamente (`intensity²`). Estrellas más intensas son más brillantes y tienen erupciones más violentas. |
| **`temperature`** | `f32` | `[0.0, 1.0]` | Factor de temperatura estelar. `0.0 - 0.3`: Estrella roja (fría, tipo M/K). `0.3 - 0.6`: Sol amarillo (medio, tipo G/F). `0.6 - 1.0`: Estrella azul (caliente, tipo B/O). | Teclas `T` (incrementar) / `G` (decrementar) | **Color dominante**: Controla el gradiente RGB. Temperatura baja = rojo dominante. Temperatura alta = azul dominante. No afecta intensidad, solo tono de color. |

**Uso de Uniforms en el Shader:**

```rust
pub fn star_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let time = uniforms.time;
    
    // 1. Calcular intensidad local con ruido
    let local_intensity = calculate_noise_intensity(pos, time);
    
    // 2. Aplicar multiplicador global
    let final_intensity = local_intensity * uniforms.intensity;
    
    // 3. Mapear a color según temperatura
    let base_color = temperature_to_color(final_intensity, uniforms.temperature);
    
    // 4. Calcular emisión
    let emission = (final_intensity * final_intensity) * 2.0;
    
    // 5. Color final emitido
    base_color * Vector3::new(0.8 * emission, 0.7 * emission, 0.6 * emission)
}
```

**Presets (Teclas 1, 2, 3):**
- **Preset 1** (Sol Amarillo): `intensity = 1.0`, `temperature = 0.5`
- **Preset 2** (Gigante Roja): `intensity = 1.2`, `temperature = 0.15`
- **Preset 3** (Estrella Azul): `intensity = 1.5`, `temperature = 0.9`

### Animación Temporal

Todos los efectos usan `uniforms.time` para animación continua:

```rust
// Turbulencia animada
fbm_noise_3d(
    pos.x * scale + time * 0.1,    // Velocidad X
    pos.y * scale + time * 0.15,   // Velocidad Y
    pos.z * scale + time * 0.12,   // Velocidad Z
    6
)

// Pulsación periódica
let pulse = (time * 1.5).sin() * 0.5 + 0.5;
```

### Vertex Shader (Corona Solar)

```rust
pub fn vertex_displacement(position: Vector3, time: f32) -> Vector3 {
    // Ruido para corona
    let corona_noise = perlin_noise_3d(...);
    
    // Ruido para prominencias
    let prominence = fbm_noise_3d(...);
    
    // Desplazamiento radial
    let displacement = corona_noise * 0.08 + prominence * 0.12;
    
    // Aplicar en dirección normal (hacia afuera)
    position + normal * displacement
}
```

### Gradiente de Temperatura

Basado en clasificación estelar real:

| Temperatura | Tipo | Color | Ejemplos |
|-------------|------|-------|----------|
| `< 0.3` | M, K | Rojo/Naranja | Betelgeuse, Antares |
| `0.3 - 0.6` | G, F | Amarillo | Sol, Alfa Centauri A |
| `> 0.6` | A, B, O | Blanco/Azul | Rigel, Sirio |

```rust
fn temperature_to_color(intensity: f32, temp_factor: f32) -> Vector3 {
    if temp_factor < 0.3 {
        // Estrella roja: R alto, G bajo, B muy bajo
        Vector3::new(0.9, 0.2, 0.05)
    } else if temp_factor < 0.6 {
        // Sol amarillo: R alto, G alto, B medio
        Vector3::new(0.95, 0.8, 0.3)
    } else {
        // Estrella azul: todos altos, B dominante
        Vector3::new(0.7, 0.85, 0.95)
    }
}
```
---

## 📁 Estructura del Proyecto

```
star_shader/
├── Cargo.toml              # Dependencias (raylib, tobj, noise)
├── README.md               # Esta documentación completa
├── .gitignore
│
├── assets/
│   └── sphere.obj          # Modelo 3D de esfera
│
├── screenshots/            # Capturas y GIF (por generar)
│   ├── yellow_sun.png
│   ├── red_giant.png
│   ├── blue_star.png
│   └── star_animation.gif
│
└── src/
    ├── main.rs             # Loop principal + UI
    ├── obj.rs              # Cargador de .obj
    └── shader.rs           # Shaders (star + vertex)
```

---

## 👨‍💻 Autor
Ernesto David Ascencio Ramírez 23009
Laboratorio de Gráficas por Computador  
Universidad del Valle de Guatemala  
Noviembre 2025
