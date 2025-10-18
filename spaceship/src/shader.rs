use raylib::math::Vector3;
use std::f32::consts::PI;

pub struct Fragment {
    pub world_position: Vector3,
    pub color: Vector3,
}

pub struct Uniforms {
    pub time: f32,
}

impl Uniforms {
    pub fn new(time: f32) -> Self {
        Uniforms { time }
    }
}

// receives fragment -> returns color
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let base_color = fragment.color;
    let time = uniforms.time;

    let angle = pos.x.atan2(pos.z) + time;
    let hue = (angle / (2.0 * PI)) % 1.0;

    // Usar sin² para colores más saturados y brillantes
    let r = (hue * 5.0).sin().abs().powf(0.5);
    let g = (hue * 5.0 + 2.0).sin().abs().powf(0.5); 
    let b = (hue * 5.0 + 4.0).sin().abs().powf(0.5);

    let pattern_color = Vector3::new(r, g, b);

    // Mezclar más del patrón para colores más vibrantes
    // Y agregar un mínimo de brillo
    let mixed = base_color * 0.2 + pattern_color * 0.8;
    
    Vector3::new(
        mixed.x.max(0.1),  // Mínimo de brillo
        mixed.y.max(0.1),
        mixed.z.max(0.1)
    )
}
