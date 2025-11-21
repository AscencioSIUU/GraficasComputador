// Shader de Mercurio - CELLULAR NOISE (Worley/Cellular) + CORONA DOMAIN WARPING

use raylib::math::Vector3;
use super::common::{Fragment, Vector3Ext, cellular_noise_3d, perlin_noise_3d, fbm_noise_3d};
use super::advanced_noise::domain_warp_3d;

pub fn vertex_displacement_mercury(position: Vector3, time: f32) -> Vector3 {
    let t = time as f64;
    
    // Cráteres simplificados (1 noise call en lugar de 2)
    let crater_noise = perlin_noise_3d(
        position.x as f64 * 10.0,
        position.y as f64 * 10.0,
        position.z as f64 * 10.0
    );
    
    // CORONA: Domain warping para exosfera SUAVE
    let corona_warp = domain_warp_3d(
        position.x as f64 * 8.0,
        position.y as f64 * 8.0,
        position.z as f64 * 8.0,
        t * 0.6
    ) as f32;
    
    let displacement = (crater_noise.abs() * 0.04 + corona_warp.abs() * 0.3) as f32;
    
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

pub fn cellular_planet_shader(fragment: &Fragment, _t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    let cellular = cellular_noise_3d(pos.x as f64 * 6.0, pos.y as f64 * 6.0, pos.z as f64 * 6.0);
    let cellular_pattern = ((cellular + 1.0) * 0.5) as f32;
    
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(2.5) * 0.5; // Aumentado para corona más visible
    
    // Corona AZUL para Aeon
    let rim_color = Vector3::new(0.2, 0.5, 1.0); // Color azul brillante
    
    let heat_boost = 0.1; // Reducido para mantener el azul oscuro
    let color_mod = Vector3::new(
        cellular_pattern * 0.3,
        cellular_pattern * 0.4,
        cellular_pattern * 0.5 + heat_boost,
    );
    
    Vector3::new(
        ((base_color.x + color_mod.x) * (0.2 + diff * 0.8) + rim * rim_color.x).clamp(0.0, 1.0),
        ((base_color.y + color_mod.y) * (0.2 + diff * 0.8) + rim * rim_color.y).clamp(0.0, 1.0),
        ((base_color.z + color_mod.z) * (0.2 + diff * 0.8) + rim * rim_color.z).clamp(0.0, 1.0),
    )
}
