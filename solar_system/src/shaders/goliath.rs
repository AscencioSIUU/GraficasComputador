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
    
    // CORONA: Value noise para gases exteriores del gigante gaseoso
    let corona_value = value_noise_3d(
        position.x as f64 * 4.0 + t * 0.5,
        position.y as f64 * 4.0 + t * 0.6,
        position.z as f64 * 4.0 + t * 0.4
    ) as f32;
    
    let displacement = (storm_noise.abs() * 0.1 + turbulence.abs() * 0.09 + corona_value.abs() * 0.8) as f32;
    
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
    
    // Múltiples capas de Perlin Noise a diferentes escalas
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
    
    // Combinar capas de ruido
    let combined_noise = (
        (large_noise + 1.0) * 0.5 * 0.5 +
        (medium_noise + 1.0) * 0.5 * 0.3 +
        (detail_noise + 1.0) * 0.5 * 0.2
    ) as f32;
    
    // Iluminación
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    
    let diff = fragment.normal.dot(light_dir).max(0.0);
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(3.0) * 0.4;
    
    // Colores variados basados en el ruido (bandas de gas gigante morado)
    let band_color = if combined_noise > 0.65 {
        // Bandas azul-violeta brillante (tormentas eléctricas)
        Vector3::new(0.4, 0.6, 1.0)
    } else if combined_noise > 0.55 {
        // Bandas moradas claras con tinte rosado
        Vector3::new(0.8, 0.5, 0.95)
    } else if combined_noise > 0.45 {
        // Bandas moradas medias
        Vector3::new(0.5, 0.3, 0.8)
    } else if combined_noise > 0.35 {
        // Bandas púrpura oscuro con tinte rojizo
        Vector3::new(0.4, 0.15, 0.65)
    } else {
        // Bandas muy oscuras (sombras profundas)
        Vector3::new(0.2, 0.05, 0.4)
    };
    
    // Mezclar con el color base para mayor variación
    let final_color = Vector3::new(
        base_color.x * 0.3 + band_color.x * 0.7,
        base_color.y * 0.3 + band_color.y * 0.7,
        base_color.z * 0.3 + band_color.z * 0.7,
    );
    
    // Aplicar iluminación
    Vector3::new(
        (final_color.x * (0.3 + diff * 0.7) + rim).clamp(0.0, 1.0),
        (final_color.y * (0.3 + diff * 0.7) + rim * 0.8).clamp(0.0, 1.0),
        (final_color.z * (0.3 + diff * 0.7) + rim * 0.6).clamp(0.0, 1.0),
    )
}
