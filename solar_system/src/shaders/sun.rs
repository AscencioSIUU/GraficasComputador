// Shader del Sol - Estrella con corona 3D optimizada

use raylib::math::Vector3;
use super::common::{Fragment, Uniforms, fbm, perlin_noise_3d, fbm_noise_3d};

pub fn vertex_displacement(position: Vector3, time: f32) -> Vector3 {
    let t = time as f64;
    
    // Corona solar - MUY PRONUNCIADA para que se vea claramente
    let corona_noise = perlin_noise_3d(
        position.x as f64 * 4.0 + t * 0.6,
        position.y as f64 * 4.0 + t * 0.8,
        position.z as f64 * 4.0 + t * 0.7
    );
    
    // Prominencias solares - grandes llamaradas
    let prominence = fbm_noise_3d(
        position.x as f64 * 6.0,
        position.y as f64 * 6.0 + t * 1.2,
        position.z as f64 * 6.0,
        3  // Reducido para performance
    );
    
    // Desplazamiento MUCHO MÁS GRANDE para corona visible (aumentado 4-5x)
    let displacement = (corona_noise.abs() * 1.0 + prominence.abs() * 1.2) as f32;
    
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

pub fn star_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    
    // Paleta de colores roja intensa
    let core_color = Vector3::new(1.0, 0.15, 0.0);      // Rojo intenso nuclear
    let mid_color = Vector3::new(1.0, 0.4, 0.05);       // Naranja rojizo
    let outer_color = Vector3::new(1.0, 0.6, 0.2);      // Naranja más claro
    let corona_color = Vector3::new(1.0, 0.05, 0.0);    // Rojo puro para corona
    
    // Múltiples capas de ruido
    let scale1 = 0.8;
    let scale2 = 1.5;
    let scale3 = 3.0;
    let time = uniforms.time * 0.3;
    
    // Noise principal (manchas solares)
    let noise1 = fbm(pos.x * scale1 + time * 0.5, pos.y * scale1 - time * 0.3, pos.z * scale1, 5);
    let noise2 = fbm(pos.x * scale2 - time * 0.7, pos.y * scale2, pos.z * scale2 + time * 0.4, 4);
    let noise3 = fbm(pos.x * scale3 + time * 1.2, pos.y * scale3 - time * 0.9, pos.z * scale3, 3);
    
    // Distancia desde el centro
    let dist_from_center = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt();
    let normalized_dist = (dist_from_center - 11.0).max(0.0) / 2.0;
    
    let combined_noise = noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2;
    
    // Interpolar colores
    let mut base_color = if combined_noise < 0.3 {
        let t = combined_noise / 0.3;
        core_color.lerp(mid_color, t)
    } else if combined_noise < 0.7 {
        let t = (combined_noise - 0.3) / 0.4;
        mid_color.lerp(outer_color, t)
    } else {
        outer_color
    };
    
    // Corona radiactiva
    let corona_intensity = (normalized_dist * 2.0).min(1.0);
    if corona_intensity > 0.0 {
        let corona_noise = fbm(pos.x * 2.0 + time * 2.0, pos.y * 2.0, pos.z * 2.0 - time * 1.5, 3);
        let pulse = (uniforms.time * 2.0).sin() * 0.3 + 0.7;
        let corona_strength = corona_intensity * (corona_noise * 0.5 + 0.5) * pulse;
        
        base_color = base_color.lerp(corona_color, corona_strength * 0.6);
        base_color = base_color * (1.0 + corona_strength * 1.5);
    }
    
    // Brillo y pulsación
    let brightness = 1.3 + combined_noise * 0.4;
    base_color = base_color * brightness;
    
    let pulse_factor = 1.0 + (uniforms.time * 1.5).sin() * 0.08;
    base_color = base_color * pulse_factor;
    
    Vector3::new(base_color.x.min(2.0), base_color.y.min(1.5), base_color.z.min(1.0))
}
