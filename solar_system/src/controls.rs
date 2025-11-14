// controls.rs - Sistema de controles de la nave espacial

use raylib::prelude::*;

pub struct ShipControls {
    pub forward: f32,     // W/S
    pub lateral_x: f32,   // A/D (movimiento absoluto en eje X mundial)
    pub rotation: f32,    // Flechas izq/der
    pub vertical: f32,    // SPACE/SHIFT
    pub nitro_active: bool,
}

impl ShipControls {
    pub fn new() -> Self {
        ShipControls {
            forward: 0.0,
            lateral_x: 0.0,
            rotation: 0.0,
            vertical: 0.0,
            nitro_active: false,
        }
    }

    /// Captura los inputs del teclado y los convierte en comandos de movimiento
    pub fn update(&mut self, rl: &RaylibHandle) {
        // Reset inputs
        self.forward = 0.0;
        self.lateral_x = 0.0;
        self.rotation = 0.0;
        self.vertical = 0.0;
        self.nitro_active = false;

        // W/S: Movimiento adelante/atrás (relativo a cámara)
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_W) {
            self.forward += 1.0;
            println!("⌨️  KEY_W pressed → forward = {:.1}", self.forward);
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_S) {
            self.forward -= 1.0;
            println!("⌨️  KEY_S pressed → forward = {:.1}", self.forward);
        }

        // A/D: Movimiento ABSOLUTO en eje X (NO relativo a cámara)
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_A) {
            self.lateral_x -= 1.0;
            println!("⌨️  KEY_A pressed → lateral_x = {:.1} (←X)", self.lateral_x);
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_D) {
            self.lateral_x += 1.0;
            println!("⌨️  KEY_D pressed → lateral_x = {:.1} (→X)", self.lateral_x);
        }

        // Flechas LEFT/RIGHT: Rotación (giro en su eje)
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT) {
            self.rotation -= 1.0;
            println!("⌨️  KEY_LEFT pressed → rotation = {:.1}", self.rotation);
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_RIGHT) {
            self.rotation += 1.0;
            println!("⌨️  KEY_RIGHT pressed → rotation = {:.1}", self.rotation);
        }

        // SPACE/SHIFT: Movimiento vertical
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_SPACE) {
            self.vertical += 1.0;
            println!("⌨️  KEY_SPACE pressed → vertical = {:.1}", self.vertical);
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT_SHIFT) 
            || rl.is_key_down(raylib::consts::KeyboardKey::KEY_RIGHT_SHIFT) {
            self.vertical -= 1.0;
            println!("⌨️  KEY_SHIFT pressed → vertical = {:.1}", self.vertical);
        }

        // X: Nitro boost
        self.nitro_active = rl.is_key_down(raylib::consts::KeyboardKey::KEY_X);
        if self.nitro_active {
            println!("⌨️  KEY_X pressed → nitro_active = true");
        }
    }

    /// Convierte los inputs a cambios de velocidad en 3D.
    /// 
    /// yaw: ángulo de orientación de la cámara (en radianes)
    /// thrust: aceleración a aplicar
    /// Returns: (vel_x, vel_z, vel_y) - cambios de velocidad en 3D
    pub fn apply_to_velocity(&self, yaw: f32, thrust: f32) -> (f32, f32, f32) {
        // Vector adelante: en dirección yaw (W/S)
        let forward_x = yaw.sin();
        let forward_z = yaw.cos();
        
        // Movimiento W/S (relativo a cámara)
        let mut vel_x = forward_x * self.forward * thrust;
        let mut vel_z = forward_z * self.forward * thrust;
        
        // Movimiento A/D (ABSOLUTO en eje X mundial, NO relativo a cámara)
        vel_x += self.lateral_x * thrust;
        
        let vel_y = self.vertical * thrust;
        
        // Debug detallado
        if self.forward != 0.0 || self.lateral_x != 0.0 || self.vertical != 0.0 {
            println!("🎮 apply_to_velocity():");
            println!("   yaw={:.1}° | forward={:.1} | lateral_x={:.1} | vertical={:.1}", 
                     yaw.to_degrees(), self.forward, self.lateral_x, self.vertical);
            println!("   velocity=({:.2}, {:.2}, {:.2})", vel_x, vel_z, vel_y);
        }
        
        (vel_x, vel_z, vel_y)
    }
}
