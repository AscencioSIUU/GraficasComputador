# Solar System — Software Renderer

Proyecto que integra los trabajos previos (`star_shader`, `planet_shaders`, `spaceship`) para crear una simulación del sistema solar usando un renderer por software.

Características principales
- Sol renderizado con ruido procedural (Perlin/FBM) y emisión.
- Varios planetas en el plano eclíptico con órbitas circulares y rotación axial.
- Cámara / nave que se mueve sobre el plano eclíptico y puede hacer "warp" instantáneo a cuerpos.
- Colisiones simples para evitar atravesar planetas.

Cómo ejecutar
1. Coloca `sphere.obj` en `solar_system/assets/` o asegúrate que exista en `../planet_shaders/assets/sphere.obj`.
2. Coloca `spaceship.obj` en `../spaceship/spaceship.obj` si deseas la nave modelada.
3. Ejecuta:

```bash
cd solar_system
cargo run --release
```

Controles
- **W/S/A/D**: mover la nave (adelante/atrás/izquierda/derecha)
- **Flechas izquierda/derecha**: rotar la vista
- **V**: cambiar vista (top-down → side → third-person)
- **Teclas 1..6**: warpear a cada cuerpo (animated warp activable con R)
- **T**: PAUSAR/DESPAUSAR el juego
- **SPACE**: NITRO BOOST (acelerar más rápido)
- **R**: alternar warp animado/instantáneo
- **ESC**: salir

Colisiones: siempre activas (no se puede desactivar)

Requisitos para entrega
- Repositorio GitHub con el proyecto en `solar_system/` (ordenado y sin código generado).
- README que explique cómo ejecutar y los controles (este archivo).
- Video o GIF corto mostrando la cámara explorando el sistema. Inclúyelo en `README.md` o en `screenshots/`.

Video / GIF
Incluye aquí el GIF o inserta el MP4 (por ejemplo con GitHub raw):

![Demo GIF](screenshots/solar_animation.gif)

Notas técnicas (ruido y uniforms)
- El Sol usa FBM (fractal brownian motion) y Perlin para crear turbulencia y manchas. El resultado se normaliza a [0,1] y se aplica un factor de emisión cuadrático para simular picos brillantes.
- Parámetros (uniforms) disponibles en el shader:
  - `time`: tiempo en segundos (animación continua).
  - `intensity`: intensidad global de la estrella (multiplica la emisión).
  - `temperature`: factor (0..1) que mezcla tonos rojo→amarillo→azul.

Funciones de ruido documentadas
- `perlin_noise_3d(x,y,z)`: valor en [-1,1] (se normaliza con `(v+1)*0.5` en los shaders).
- `fbm_noise_3d(x,y,z,octaves)`: suma de octavas de Perlin para detalle multi-escala.
- `turbulence_noise_3d(...)`: versión con valores absolutos para patrones más contrastados (útil en lava/erupciones).

Estructura del repositorio
```
solar_system/
├── Cargo.toml
├── README.md
├── assets/
│   └── sphere.obj   # required
└── src/
    ├── main.rs
    ├── obj.rs
    └── shader.rs
```

Créditos
- Basado en los proyectos `star_shader`, `planet_shaders` y `spaceship` dentro de este workspace.


