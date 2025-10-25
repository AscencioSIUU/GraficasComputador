use raylib::prelude::*;
#[derive(Clone, Copy)]
pub struct Player {
    pub pos: Vector2, // world-space (pixels)
    pub a: f32,       // angle (rads)
    pub fov: f32,
    pub health: i32,      // Salud actual
    pub max_health: i32,  // Salud máxima
    pub shot_effect_timer: i32, // frames para mostrar efecto de disparo
}
impl Player {
    pub fn new(pos: Vector2) -> Self {
        Self {
            pos,
            a: 0.0,
            fov: std::f32::consts::PI / 3.0,
            health: 100,
            max_health: 100,
            shot_effect_timer: 0,
        }
    }
    
    // Método para recibir daño
    pub fn take_damage(&mut self, damage: i32) {
        self.health = (self.health - damage).max(0);
    }
    
    // Método para curar
    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }
    
    // Verificar si está vivo
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    /// Activar efecto de disparo
    pub fn trigger_shot_effect(&mut self) {
        self.shot_effect_timer = 8; // Mostrar por 8 frames (~0.13s a 60fps)
    }
    
    /// Actualizar timer del efecto
    pub fn update_shot_effect(&mut self) {
        if self.shot_effect_timer > 0 {
            self.shot_effect_timer -= 1;
        }
    }
}
