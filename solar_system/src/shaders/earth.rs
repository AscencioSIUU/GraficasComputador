// Shader de la Tierra - VORONOI NOISE (océanos y continentes) + CORONA WORLEY

use raylib::math::Vector3;
use super::common::{Fragment, Vector3Ext, voronoi_noise_3d, perlin_noise_3d, fbm_noise_3d};
use super::advanced_noise::worley_noise_3d;

pub fn vertex_displacement_earth(position: Vector3, time: f32) -> Vector3 {
    let t = time as f64;
    
    // Montañas y océanos optimizados
    let terrain_noise = voronoi_noise_3d(
        position.x as f64 * 6.0,
        position.y as f64 * 6.0,
        position.z as f64 * 6.0
    );
    
    let mountains = fbm_noise_3d(
        position.x as f64 * 10.0,
        position.y as f64 * 10.0,
        position.z as f64 * 10.0,
        2  // Reducido de 3 a 2
    );
    
    // CORONA: Worley noise para atmósfera celular SUAVE
    let corona_worley = worley_noise_3d(
        position.x as f64 * 10.0 + t * 0.3,
        position.y as f64 * 10.0 + t * 0.4,
        position.z as f64 * 10.0 + t * 0.2
    ) as f32;
    
    let displacement = (terrain_noise.abs() * 0.05 + mountains.abs() * 0.04 + corona_worley * 0.35) as f32;
    
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

pub fn voronoi_planet_shader(fragment: &Fragment, _t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    let voronoi = voronoi_noise_3d(pos.x as f64 * 5.0, pos.y as f64 * 5.0, pos.z as f64 * 5.0);
    let voronoi_pattern = ((voronoi + 1.0) * 0.5) as f32;
    
    let detail = perlin_noise_3d(pos.x as f64 * 10.0, pos.y as f64 * 10.0, pos.z as f64 * 10.0);
    let combined = voronoi_pattern * 0.7 + ((detail + 1.0) * 0.5) as f32 * 0.3;
    
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(2.5) * 0.45;
    
    let heat_shift = 0.2;
    let color_shift = Vector3::new(
        combined * 0.4 + heat_shift,
        combined * 0.4 + heat_shift * 0.6,
        combined * 0.3 - 0.1,
    );
    
    Vector3::new(
        ((base_color.x * 1.2 + color_shift.x) * (0.2 + diff * 0.8) + rim).clamp(0.0, 1.0),
        ((base_color.y * 1.1 + color_shift.y) * (0.2 + diff * 0.8) + rim * 0.8).clamp(0.0, 1.0),
        ((base_color.z * 0.9 + color_shift.z) * (0.2 + diff * 0.8) + rim * 0.6).clamp(0.0, 1.0),
    )
}
