// Shader de Goliath - Planeta gigante con Perlin Noise complejo + CORONA VALUE NOISE

use raylib::math::Vector3;
use super::common::{Fragment, Vector3Ext, fbm_noise_3d, perlin_noise_3d};
use super::advanced_noise::value_noise_3d;

pub fn vertex_displacement_goliath(position: Vector3, time: f32) -> Vector3 {
    let t = time as f64;
    
    // Tormentas de gas optimizadas
    let storm_noise = perlin_noise_3d(
        position.x as f64 * 5.0 + t * 0.3,
        position.y as f64 * 5.0,
        position.z as f64 * 5.0 + t * 0.25
    );
    
    let turbulence = fbm_noise_3d(
        position.x as f64 * 7.0,
        position.y as f64 * 7.0 + t * 0.5,
        position.z as f64 * 7.0,
        3  // Reducido de 5 a 3
    );
    
    // CORONA: Value noise para gases exteriores SUAVES del gigante gaseoso
    let corona_value = value_noise_3d(
        position.x as f64 * 5.0 + t * 0.5,
        position.y as f64 * 5.0 + t * 0.6,
        position.z as f64 * 5.0 + t * 0.4
    ) as f32;
    
    let displacement = (storm_noise.abs() * 0.08 + turbulence.abs() * 0.06 + corona_value.abs() * 0.5) as f32;
    
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

pub fn planet_shader(fragment: &Fragment, t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;

    // Multi-scale noise
    let large_noise = perlin_noise_3d(
        pos.x as f64 * 1.5 + t as f64 * 0.1,
        pos.y as f64 * 1.5,
        pos.z as f64 * 1.5
    );

    let medium_noise = perlin_noise_3d(
        pos.x as f64 * 4.0 + t as f64 * 0.2,
        pos.y as f64 * 4.0,
        pos.z as f64 * 4.0
    );

    let detail_noise = fbm_noise_3d(
        pos.x as f64 * 8.0,
        pos.y as f64 * 8.0 + t as f64 * 0.15,
        pos.z as f64 * 8.0,
        4
    );

    let combined_noise = (
        (large_noise + 1.0) * 0.5 * 0.5 +
        (medium_noise + 1.0) * 0.5 * 0.3 +
        (detail_noise + 1.0) * 0.5 * 0.2
    ) as f32;

    // Lighting
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();

    let diff = fragment.normal.dot(light_dir).max(0.0);
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(1.8); // Más suave para aura más amplia

    // Purple bands NEÓN con toques NEGROS (morado brillante intenso con negro)
    let band_color = if combined_noise > 0.75 {
        Vector3::new(1.0, 0.3, 1.0)   // neón magenta ultra brillante
    } else if combined_noise > 0.6 {
        Vector3::new(0.95, 0.2, 1.0)  // neón púrpura-magenta intenso
    } else if combined_noise > 0.45 {
        Vector3::new(0.6, 0.1, 0.85)  // neón morado medio
    } else if combined_noise > 0.3 {
        Vector3::new(0.3, 0.0, 0.5)   // morado oscuro con negro
    } else if combined_noise > 0.15 {
        Vector3::new(0.15, 0.0, 0.25) // casi negro con toque morado
    } else {
        Vector3::new(0.05, 0.0, 0.1)  // negro profundo con mínimo morado
    };

    // Mix base planet color with band color - ultra saturado con negros
    let mixed_purple = Vector3::new(
        base_color.x * 0.2 + band_color.x * 0.8,
        base_color.y * 0.2 + band_color.y * 0.8,
        base_color.z * 0.2 + band_color.z * 0.8,
    );

    // Corona tint NEÓN EXPANDIDO (brillo magenta mega intenso)
    let corona_tint = Vector3::new(1.0, 0.2, 1.0); // neón magenta puro

    let lit = Vector3::new(
        mixed_purple.x * (0.2 + diff * 0.8),
        mixed_purple.y * (0.2 + diff * 0.8),
        mixed_purple.z * (0.2 + diff * 0.8),
    );

    // Aura expandida mucho más grande y brillante
    let with_rim = Vector3::new(
        (lit.x + rim * corona_tint.x * 1.5).clamp(0.0, 2.0), // Aura más intensa y expandida
        (lit.y + rim * corona_tint.y * 1.5).clamp(0.0, 2.0),
        (lit.z + rim * corona_tint.z * 1.5).clamp(0.0, 2.0),
    );

    with_rim
}
