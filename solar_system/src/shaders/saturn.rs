// Shader de Saturno - Gigante gaseoso con bandas suaves y anillos

use raylib::math::Vector3;
use super::common::{Fragment, Uniforms, simplex_noise_3d};

pub fn vertex_displacement_saturn(position: Vector3, time: f32) -> Vector3 {
    let t = time as f64;
    
    // Atmósfera muy suave y sutil
    let atmosphere = simplex_noise_3d(
        position.x as f64 * 3.0 + t * 0.3,
        position.y as f64 * 3.0,
        position.z as f64 * 3.0 + t * 0.3
    );
    
    // Desplazamiento muy pequeño para mantener superficie lisa
    let displacement = (atmosphere.abs() * 0.15) as f32;
    
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

pub fn saturn_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    
    // Paleta de colores amarillo/beige suave como Saturno
    let band1 = Vector3::new(0.95, 0.90, 0.70);  // Amarillo pálido
    let band2 = Vector3::new(0.90, 0.82, 0.60);  // Beige claro
    let band3 = Vector3::new(0.85, 0.75, 0.55);  // Beige medio
    let band4 = Vector3::new(0.78, 0.68, 0.50);  // Beige oscuro
    let band5 = Vector3::new(0.70, 0.60, 0.45);  // Marrón claro
    
    let time = uniforms.time * 0.2;
    
    // Usar coordenada Y para crear bandas horizontales suaves
    // Normalizar Y entre -1 y 1
    let y_norm = pos.y / 13.2; // Dividir por el tamaño del planeta
    
    // Agregar variación sutil con noise
    let noise_variation = simplex_noise_3d(
        pos.x as f64 * 2.0 + time as f64,
        pos.y as f64 * 2.0,
        pos.z as f64 * 2.0 + time as f64
    ) as f32;
    
    // Crear bandas basadas en latitud con transiciones suaves
    let band_position = (y_norm + noise_variation * 0.1 + 1.0) / 2.0; // 0 a 1
    
    let color = if band_position < 0.2 {
        let t = band_position / 0.2;
        band1.lerp(band2, t)
    } else if band_position < 0.4 {
        let t = (band_position - 0.2) / 0.2;
        band2.lerp(band3, t)
    } else if band_position < 0.6 {
        let t = (band_position - 0.4) / 0.2;
        band3.lerp(band4, t)
    } else if band_position < 0.8 {
        let t = (band_position - 0.6) / 0.2;
        band4.lerp(band5, t)
    } else {
        let t = (band_position - 0.8) / 0.2;
        band5.lerp(band1, t)
    };
    
    // Agregar ligera variación de turbulencia para realismo
    let turbulence = simplex_noise_3d(
        pos.x as f64 * 5.0 + time as f64 * 0.5,
        pos.y as f64 * 5.0,
        pos.z as f64 * 5.0 + time as f64 * 0.5
    ) as f32 * 0.05;
    
    Vector3::new(
        (color.x + turbulence).max(0.0).min(1.0),
        (color.y + turbulence).max(0.0).min(1.0),
        (color.z + turbulence).max(0.0).min(1.0),
    )
}
