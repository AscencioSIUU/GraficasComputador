# 🎮 Raytracing Diorama - Proyecto Final

## Descripción
Diorama 3D tipo Minecraft renderizado con raytracing, implementando múltiples efectos avanzados de iluminación y materiales.

## ✨ Características Implementadas

### 🎯 Puntos Principales
- ✅ **[30 pts] Escena compleja**: Diorama con múltiples estructuras, terreno, agua, portal
- ✅ **[20 pts] Visualmente atractiva**: Texturas de Minecraft, efectos de portal, iluminación dinámica
- ✅ **[20 pts] Optimización FPS**: Multithreading con rayon
- ✅ **[15 pts] Day/Night Cycle**: Sol que se mueve y cambia la iluminación
- ✅ **[10 pts] Texturas animadas**: Agua y portal con animación
- ✅ **[15 pts] Threads**: Paralelización del rendering
- ✅ **[10 pts] Controles de cámara**: Rotación, zoom, movimiento suave
- ✅ **[25 pts] Materiales diversos**: 5+ tipos (grass, stone, water, glass, torch)
- ✅ **[10 pts] Refracción**: Agua y vidrio con refracción física
- ✅ **[5 pts] Reflexión**: Agua refleja el entorno
- ✅ **[10 pts] Skybox**: Cielo con texturas día/noche
- ✅ **[20 pts] Materiales emisivos**: Antorchas que emiten luz
- ✅ **[20 pts] Efectos portal**: Portal estilo Minecraft con distorsión

**Total: 210+ puntos disponibles**

## 🎮 Controles

- **W/S**: Acercar/Alejar cámara
- **A/D**: Rotar cámara horizontalmente
- **Q/E**: Rotar cámara verticalmente
- **Espacio**: Pausar/Reanudar ciclo día/noche
- **T**: Acelerar tiempo
- **R**: Reset cámara
- **ESC**: Salir

## 🛠️ Compilación y Ejecución

```bash
# Compilar y ejecutar en modo debug
cargo run

# Compilar y ejecutar optimizado (mejor FPS)
cargo run --release
```

## 📦 Dependencias

- **Rust**: 1.70+
- **raylib**: Rendering y manejo de ventanas
- **rayon**: Paralelización para mejor rendimiento

## 🏗️ Estructura del Proyecto

```
raytracing/
├── src/
│   ├── main.rs          # Loop principal y controles
│   ├── framebuffer.rs   # Buffer de píxeles
│   ├── raytracer.rs     # Motor de raytracing
│   └── materials.rs     # Sistema de materiales
├── assets/              # Texturas
└── Cargo.toml
```

## 🎨 Materiales Implementados

1. **Grass Block**: Albedo medio, sin reflexión
2. **Stone**: Albedo bajo, levemente especular
3. **Water**: Transparente, refractivo, reflectivo, animado
4. **Glass**: Transparente, altamente refractivo
5. **Torch**: Emisivo, emite luz naranja
6. **Portal**: Emisivo, efecto especial animado

## 🔬 Técnicas de Raytracing

- Ray-sphere intersection
- Ray-triangle intersection con coordenadas baricéntricas
- Reflexión recursiva
- Refracción con Ley de Snell
- Soft shadows
- Ambient occlusion básico
- Iluminación global simplificada
- Emisión de luz desde materiales

## 📹 Video Demo

[Enlace al video en YouTube aquí]

## 👨‍💻 Autor

Proyecto desarrollado para el curso de Gráficas por Computador

---
⭐ No se usaron librerías externas más allá de raylib (requisito del curso)
