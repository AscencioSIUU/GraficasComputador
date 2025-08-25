use raylib::prelude::*;
#[derive(Clone, Copy)]
pub struct Player {
    pub pos: Vector2, // world-space (pixels)
    pub a: f32,       // angle (rads)
    pub fov: f32,
    pub hp: i32,
}
impl Player {
    pub fn new(pos: Vector2) -> Self {
        Self { pos, a: 0.0, fov: std::f32::consts::FRAC_PI_2, hp: 100 }
    }
}
