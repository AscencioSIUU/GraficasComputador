# 🎮 Rust Raycaster - Dungeon Delver

![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)
![Raylib](https://img.shields.io/badge/Raylib-5.0-blue.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

Un shooter first-person estilo **Wolfenstein 3D** desarrollado desde cero en Rust usando raycasting puro. Explora mazmorras oscuras, recolecta monedas y derrota enemigos en este homenaje a los clásicos FPS de los 90s.

---

## 📖 Descripción

**Dungeon Delver** es un juego de disparos en primera persona inspirado en **Wolfenstein 3D**. Utiliza técnicas de raycasting para renderizar un mundo 3D en tiempo real, creando una experiencia retro pero fluida.

### 🎯 Objetivo del Juego

Tu misión es sobrevivir y progresar a través de 3 niveles:

* **Recolectar 10 monedas** por nivel
* **Eliminar enemigos** (¡te disparan!)
* **Explorar el mapa** con fog of war
* **Completar el nivel 3 para ganar**

### ✨ Características

* ✅ Raycasting puro — 3D en tiempo real
* ✅ Fog of War dinámico
* ✅ Enemigos con IA (movimiento + ataque)
* ✅ Dificultad progresiva con 3 niveles
* ✅ Sistema de daño con barras de vida
* ✅ Efectos visuales (muzzle flash, disparos)
* ✅ Render scale dinámico para mantener 60 FPS
* ✅ Música y sonidos opcionales

---

## 🎬 Video Demo

[![Gameplay Video](https://img.youtube.com/vi/NpGpvfgg7NE/maxresdefault.jpg)](https://youtu.be/NpGpvfgg7NE?si=zYu-LoOQ69L0scJZ)

👉 **[Ver video completo en YouTube](https://youtu.be/NpGpvfgg7NE?si=zYu-LoOQ69L0scJZ)**

---

## 📥 Descarga e Instalación

### ✅ Prerequisitos

* **Rust** 1.70+ → [https://rustup.rs/](https://rustup.rs/)
* **Git**

### 🛠️ Pasos

```bash
# 1️⃣ Clonar el repositorio
git clone https://github.com/tu-usuario/raycaster.git
cd raycaster

# 2️⃣ Compilar
cargo build --release

# 3️⃣ Ejecutar
cargo run --release
```

---

## 🎮 Controles

### Controles Básicos

| Tecla        | Acción          |
| ------------ | --------------- |
| W            | Avanzar         |
| S            | Retroceder      |
| A            | Girar izquierda |
| D            | Girar derecha   |
| ESPACIO      | Disparar        |
| ESC          | Pausar juego    |
| Q (en pausa) | Salir al menú   |

### Controles Extra

| Tecla | Función                                 |
| ----- | --------------------------------------- |
| M     | Mutear/Desmutear música                 |
| Z     | Aumentar calidad (reducir render scale) |
| X     | Reducir calidad (aumentar render scale) |

### Menú Principal

| Tecla | Acción                   |
| ----- | ------------------------ |
| ↑/↓   | Navegar menú             |
| ENTER | Seleccionar nivel/opción |

---

## 🕹️ Mecánicas del Juego

### Sistema de Combate

* Dispara apuntando con la mira verde + **ESPACIO**
* Daño jugador: **34 HP** por disparo
  → **3 disparos** = enemigo eliminado
* Daño enemigo: **10 HP** por disparo (70% precisión)
* Vida inicial: **100 HP** tanto jugador como enemigos

### Fog of War

* Mapa oculto inicialmente
* Se revela al explorar (radio: **400px**)
* Enemigos visibles solo tras exploración
* Áreas descubiertas permanecen iluminadas

### HUD

* **Barra de vida** (abajo izquierda)

  * 🟢 > 60%
  * 🟠 30–60%
  * 🔴 < 30%
* Minimapa (arriba izquierda)
* Contador de monedas (arriba derecha)
* FPS + info de controles
* Mira verde en el centro

### Progresión de Niveles

| Nivel   | Descripción                             |
| ------- | --------------------------------------- |
| Level 1 | Tutorial — 20 enemigos — Mapa pequeño   |
| Level 2 | Intermedio — 25 enemigos — Mapa mediano |
| Level 3 | Final — 30 enemigos — Mapa grande       |

---

## 🛠️ Tecnologías Utilizadas

* 🦀 **Rust**
* 🎮 **Raylib-rs**
* 🖼️ **Image** (texturas)
* 🎲 **Rand** (IA de enemigos)

---

## 🎨 Assets y Créditos

* Texturas inspiradas en Wolfenstein 3D / Doom
* Música: *Dungeon Delver* (música libre)
* Proyecto educativo — Gráficas Computacionales

---

## 📊 Optimización

Render scale dinámico para estabilidad:

* Inicio: **escala x3**
* Se ajusta para mantener **60 FPS**
* Manual: **Z / X**
* Fog al 50% para mejor visibilidad
* Sprites optimizados con **Z-buffer**

---

## 📝 Licencia

Este proyecto es **open-source** bajo la licencia **MIT**.
Creado con fines educativos.

---

## 👨‍💻 Autor

Ernesto David Ascencio Ramírez 23009
Proyecto desarrollado para el curso de **Gráficas Computacionales — 2025**.
Universidad del Valle de Guatemala.

---

### 🏁 ¡Disfruta el juego y sobrevive a las mazmorras! 🎯🔥

---

Si quieres, puedo también:

✅ Crear **badges adicionales** (FPS, estado del build, etc.)
✅ Agregar **screenshots del HUD y enemigos**
✅ Añadir sección **Roadmap** y **Changelog**
✅ Preparar **release tags** para descargar binarios

¿Quieres que te genere también un **icono del juego** o **splash screen** para el README? 🎨🚀
