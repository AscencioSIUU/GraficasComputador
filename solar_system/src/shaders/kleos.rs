// Shader de Kleos (Tierra) - Verde y azul con superficie lisa

use raylib::math::Vector3;
use super::common::{Fragment, Vector3Ext, simplex_noise_3d};

pub fn kleos_shader(fragment: &Fragment, t: f32, _base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    // Paleta de colores de Tierra - Verde y Azul
    let deep_ocean = Vector3::new(0.05, 0.25, 0.60);   // Azul océano profundo
    let ocean = Vector3::new(0.10, 0.35, 0.75);        // Azul océano
    let shallow = Vector3::new(0.15, 0.50, 0.85);      // Azul claro
    let coast = Vector3::new(0.25, 0.65, 0.40);        // Verde-azul costa
    let land = Vector3::new(0.25, 0.70, 0.30);         // Verde tierra
    let forest = Vector3::new(0.20, 0.55, 0.25);       // Verde bosque
    
    let time = t * 0.15;
    
    // Usar coordenada Y para crear bandas horizontales suaves (latitudes)
    let y_norm = pos.y / 6.0; // Normalizar según tamaño de Kleos
    
    // Agregar variación sutil con noise para simular continentes
    let noise_variation = simplex_noise_3d(
        pos.x as f64 * 2.5 + time as f64,
        pos.y as f64 * 2.5,
        pos.z as f64 * 2.5 + time as f64
    ) as f32;
    
    // Crear "continentes" basados en latitud con transiciones suaves
    let band_position = (y_norm + noise_variation * 0.15 + 1.0) / 2.0; // 0 a 1
    
    // Bandas de color que mezclan océanos y tierra suavemente
    let color = if band_position < 0.15 {
        let t = band_position / 0.15;
        deep_ocean.lerp(ocean, t)
    } else if band_position < 0.35 {
        let t = (band_position - 0.15) / 0.2;
        ocean.lerp(shallow, t)
    } else if band_position < 0.5 {
        let t = (band_position - 0.35) / 0.15;
        shallow.lerp(coast, t)
    } else if band_position < 0.7 {
        let t = (band_position - 0.5) / 0.2;
        coast.lerp(land, t)
    } else if band_position < 0.85 {
        let t = (band_position - 0.7) / 0.15;
        land.lerp(forest, t)
    } else {
        let t = (band_position - 0.85) / 0.15;
        forest.lerp(deep_ocean, t)
    };
    
    // Agregar nubes suaves y ligeras
    let cloud_noise = simplex_noise_3d(
        pos.x as f64 * 4.0 + time as f64 * 0.3,
        pos.y as f64 * 4.0,
        pos.z as f64 * 4.0 + time as f64 * 0.4
    ) as f32;
    
    let cloud_intensity = ((cloud_noise + 1.0) * 0.5).powf(3.0) * 0.2;
    let cloud_color = Vector3::new(1.0, 1.0, 1.0);
    
    let with_clouds = Vector3::new(
        color.x * (1.0 - cloud_intensity) + cloud_color.x * cloud_intensity,
        color.y * (1.0 - cloud_intensity) + cloud_color.y * cloud_intensity,
        color.z * (1.0 - cloud_intensity) + cloud_color.z * cloud_intensity
    );
    
    // Iluminación suave
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    // Rim light para atmósfera
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(3.0) * 0.25;
    let rim_color = Vector3::new(0.4, 0.7, 1.0); // Azul cielo
    
    Vector3::new(
        (with_clouds.x * (0.3 + diff * 0.7) + rim * rim_color.x).clamp(0.0, 1.0),
        (with_clouds.y * (0.3 + diff * 0.7) + rim * rim_color.y).clamp(0.0, 1.0),
        (with_clouds.z * (0.3 + diff * 0.7) + rim * rim_color.z).clamp(0.0, 1.0),
    )
}
