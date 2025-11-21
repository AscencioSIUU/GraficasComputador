// Shader de Venus - SIMPLEX NOISE (nubes ácidas) + CORONA RIDGED MULTIFRACTAL

use raylib::math::Vector3;
use super::common::{Fragment, Vector3Ext, simplex_noise_3d, perlin_noise_3d, fbm_noise_3d};
use super::advanced_noise::ridged_multifractal;

pub fn vertex_displacement_venus(position: Vector3, time: f32) -> Vector3 {
    let t = time as f64;
    
    // Atmósfera con menos complejidad
    let cloud_noise = simplex_noise_3d(
        position.x as f64 * 8.0 + t * 0.25,
        position.y as f64 * 8.0,
        position.z as f64 * 8.0 + t * 0.15
    );
    
    let turbulence = fbm_noise_3d(
        position.x as f64 * 12.0,
        position.y as f64 * 12.0 + t * 0.4,
        position.z as f64 * 12.0,
        2  // Reducido de 4 a 2
    );
    
    // CORONA: Ridged multifractal para capas de atmósfera SUAVES
    let corona_ridged = ridged_multifractal(
        position.x as f64 * 6.0 + t * 0.4,
        position.y as f64 * 6.0 + t * 0.5,
        position.z as f64 * 6.0 + t * 0.3,
        3  // octaves
    ) as f32;
    
    let displacement = (cloud_noise.abs() * 0.06 + turbulence.abs() * 0.05 + corona_ridged * 0.45) as f32;
    
    let len = (position.x * position.x + position.y * position.y + position.z * position.z).sqrt();
    let direction = if len > 0.001 {
        Vector3::new(position.x / len, position.y / len, position.z / len)
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    };
    
    Vector3::new(
        position.x + direction.x * displacement,
        position.y + direction.y * displacement,
        position.z + direction.z * displacement,
    )
}

pub fn simplex_planet_shader(fragment: &Fragment, t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    let simplex1 = simplex_noise_3d(pos.x as f64 * 4.0 + t as f64 * 0.2,
                                     pos.y as f64 * 4.0,
                                     pos.z as f64 * 4.0 + t as f64 * 0.2);
    let simplex2 = simplex_noise_3d(pos.x as f64 * 8.0, pos.y as f64 * 8.0, pos.z as f64 * 8.0);
    
    let pattern = ((simplex1 + simplex2) * 0.5 + 1.0) * 0.5;
    let swirl = (pattern * 3.14159 * 2.0).sin() * 0.3;
    
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(2.5) * 0.5;
    
    let heat_glow = 0.2;
    Vector3::new(
        ((base_color.x * 1.3 + swirl as f32 + heat_glow) * (0.25 + diff * 0.75) + rim).clamp(0.0, 1.0),
        ((base_color.y * 1.2 + swirl as f32 * 0.8 + heat_glow * 0.7) * (0.25 + diff * 0.75) + rim * 0.8).clamp(0.0, 1.0),
        ((base_color.z * 0.7 + swirl as f32 * 0.5) * (0.25 + diff * 0.75) + rim * 0.6).clamp(0.0, 1.0),
    )
}
