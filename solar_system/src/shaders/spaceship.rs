// Shader de la nave espacial - Colores iridiscentes

use raylib::math::Vector3;
use super::common::Fragment;
use std::f32::consts::PI;

pub fn spaceship_shader(fragment: &Fragment, time: f32) -> Vector3 {
    let pos = fragment.world_position;
    
    let angle = pos.x.atan2(pos.z) + time;
    let hue = (angle / (2.0 * PI)) % 1.0;

    let r = (hue * 5.0).sin().abs().powf(0.5);
    let g = (hue * 5.0 + 2.0).sin().abs().powf(0.5); 
    let b = (hue * 5.0 + 4.0).sin().abs().powf(0.5);

    let pattern_color = Vector3::new(r, g, b);
    let base_color = Vector3::new(0.8, 0.8, 0.9);
    let mixed = base_color * 0.2 + pattern_color * 0.8;
    
    Vector3::new(mixed.x.max(0.1), mixed.y.max(0.1), mixed.z.max(0.1))
}
