// Shader de Marte - PERLIN NOISE (tormentas de polvo) + CORONA FBM

use raylib::math::Vector3;
use super::common::{Fragment, Vector3Ext, perlin_noise_3d, fbm_noise_3d};
use super::advanced_noise::fbm_enhanced;

pub fn vertex_displacement_mars(position: Vector3, time: f32) -> Vector3 {
    let t = time as f64;
    
    // Cañones optimizados
    let canyon_noise = perlin_noise_3d(
        position.x as f64 * 8.0,
        position.y as f64 * 8.0,
        position.z as f64 * 8.0
    );
    
    let dunes = fbm_noise_3d(
        position.x as f64 * 11.0,
        position.y as f64 * 11.0,
        position.z as f64 * 11.0,
        2  // Reducido de 4 a 2
    );
    
    // CORONA: FBM mejorado para tormentas de polvo SUAVES
    let corona_fbm = fbm_enhanced(
        position.x as f64 * 8.0 + t * 0.4,
        position.y as f64 * 8.0 + t * 0.3,
        position.z as f64 * 8.0 + t * 0.5,
        3,      // octaves
        2.0,    // lacunarity
        0.5     // gain
    ) as f32;
    
    let displacement = (canyon_noise.abs() * 0.04 + dunes.abs() * 0.03 + corona_fbm.abs() * 0.4) as f32;
    
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

pub fn perlin_planet_shader(fragment: &Fragment, t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    let perlin1 = perlin_noise_3d(pos.x as f64 * 3.0, pos.y as f64 * 3.0, pos.z as f64 * 3.0);
    let perlin2 = perlin_noise_3d(pos.x as f64 * 7.0, pos.y as f64 * 7.0, pos.z as f64 * 7.0);
    let perlin3 = perlin_noise_3d(pos.x as f64 * 15.0, pos.y as f64 * 15.0, pos.z as f64 * 15.0);
    
    let multi_scale = ((perlin1 + 1.0) * 0.5 * 0.5 + (perlin2 + 1.0) * 0.5 * 0.3 + (perlin3 + 1.0) * 0.5 * 0.2) as f32;
    
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(2.5) * 0.5;
    
    let heat_boost = 0.3;
    let terrain = Vector3::new(
        multi_scale * 0.5 + heat_boost,
        multi_scale * 0.3 + heat_boost * 0.3,
        multi_scale * 0.1,
    );
    
    Vector3::new(
        ((base_color.x * 1.3 + terrain.x) * (0.2 + diff * 0.8) + rim).clamp(0.0, 1.0),
        ((base_color.y * 1.1 + terrain.y) * (0.2 + diff * 0.8) + rim * 0.8).clamp(0.0, 1.0),
        ((base_color.z * 0.7 + terrain.z) * (0.2 + diff * 0.8) + rim * 0.6).clamp(0.0, 1.0),
    )
}
