use raylib::math::Vector3;
use noise::{NoiseFn, Perlin, Fbm, MultiFractal, OpenSimplex, Worley};

pub struct Fragment {
    pub world_position: Vector3,
    pub normal: Vector3,
}

pub struct Uniforms {
    pub time: f32,
    pub intensity: f32,
}

impl Uniforms {
    pub fn new(time: f32, intensity: f32) -> Self {
        Uniforms { time, intensity }
    }
}

fn perlin_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    let perlin = Perlin::new(42);
    perlin.get([x, y, z]) as f32
}

fn fbm_noise_3d(x: f64, y: f64, z: f64, octaves: usize) -> f32 {
    let fbm = Fbm::<Perlin>::new(42).set_octaves(octaves);
    fbm.get([x, y, z]) as f32
}

fn simplex_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    let simplex = OpenSimplex::new(42);
    simplex.get([x, y, z]) as f32
}

fn cellular_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    let worley = Worley::new(42);
    worley.get([x, y, z]) as f32
}

fn voronoi_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    let worley = Worley::new(123); // Different seed for variation
    worley.get([x, y, z]) as f32
}

pub fn star_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    // Sol con RUIDO FRACTAL mejorado (FBM intenso) - Temperatura controlada con corona
    let pos = fragment.world_position;
    let time = uniforms.time as f64;

    // Múltiples capas de ruido fractal para efecto dramático
    let fractal1 = fbm_noise_3d(pos.x as f64 * 4.0 + time * 0.5,
                                pos.y as f64 * 4.0 + time * 0.55,
                                pos.z as f64 * 4.0 + time * 0.6,
                                8);
    
    let fractal2 = fbm_noise_3d(pos.x as f64 * 8.0 - time * 0.3,
                                pos.y as f64 * 8.0,
                                pos.z as f64 * 8.0 - time * 0.3,
                                6);
    
    let turbulence = ((fractal1 + fractal2) * 0.5 + 1.0) * 0.5;
    let fractal_intensity = turbulence.powf(0.4);

    // Manchas solares
    let sunspot_fractal = fbm_noise_3d(pos.x as f64 * 5.0 + time * 0.15,
                                       pos.y as f64 * 5.0,
                                       pos.z as f64 * 5.0 + time * 0.15,
                                       4);
    let sunspot = if sunspot_fractal < -0.1 { (sunspot_fractal.abs() * 1.8).min(0.5) } else { 0.0 };

    let mut intensity = 0.8 + fractal_intensity * 1.0 - sunspot * 0.3;
    intensity *= uniforms.intensity;
    intensity = intensity.clamp(0.3, 3.5);

    // COLORES CÁLIDOS pero no excesivos
    let color = Vector3::new(1.0, 0.85, 0.3);

    let emission = (intensity * intensity * 2.0).min(4.0);

    Vector3::new(
        (color.x * 0.6 + emission * 1.0).clamp(0.0, 1.0),
        (color.y * 0.6 + emission * 0.9).clamp(0.0, 1.0),
        (color.z * 0.5 + emission * 0.7).clamp(0.0, 1.0),
    )
}

// Goliath: planet shader (diffuse + procedural variation) - Con corona
pub fn planet_shader(fragment: &Fragment, t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    // Múltiples capas de ruido para más detalle
    let noise1 = fbm_noise_3d(pos.x as f64 * 3.0 + t as f64 * 0.15,
                              pos.y as f64 * 3.0,
                              pos.z as f64 * 3.0,
                              5);
    let noise2 = perlin_noise_3d(pos.x as f64 * 8.0, pos.y as f64 * 8.0, pos.z as f64 * 8.0);
    
    // Combinar ruidos para crear patrones más interesantes
    let combined_noise = ((noise1 + 1.0) * 0.5 * 0.7 + (noise2 + 1.0) * 0.5 * 0.3) as f32;
    
    // Iluminación mejorada con múltiples fuentes de luz
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    // Corona (rim light) - brillo en los bordes
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(2.5) * 0.55;
    
    // Variación de color cálida moderada
    let heat_factor = 0.3; // Reducido mucho
    let color_variation = Vector3::new(
        (combined_noise * 0.5 + heat_factor).clamp(-0.1, 0.6),
        (combined_noise * 0.4 + heat_factor * 0.5).clamp(-0.1, 0.5),
        (combined_noise * 0.3 - 0.1).clamp(-0.2, 0.3),
    );
    
    // Aplicar iluminación y variación con brillo moderado
    let final_color = Vector3::new(
        ((base_color.x * 1.3 + color_variation.x) * (0.2 + diff * 0.8) + rim).clamp(0.0, 1.0),
        ((base_color.y * 1.2 + color_variation.y) * (0.2 + diff * 0.8) + rim * 0.9).clamp(0.0, 1.0),
        ((base_color.z * 1.0 + color_variation.z) * (0.2 + diff * 0.8) + rim * 0.7).clamp(0.0, 1.0),
    );
    
    final_color
}

// Shader para la nave espacial (similar a spaceship/shader.rs)
pub fn spaceship_shader(fragment: &Fragment, time: f32) -> Vector3 {
    let pos = fragment.world_position;
    use std::f32::consts::PI;
    
    let angle = pos.x.atan2(pos.z) + time;
    let hue = (angle / (2.0 * PI)) % 1.0;

    // Usar sin² para colores más saturados y brillantes
    let r = (hue * 5.0).sin().abs().powf(0.5);
    let g = (hue * 5.0 + 2.0).sin().abs().powf(0.5); 
    let b = (hue * 5.0 + 4.0).sin().abs().powf(0.5);

    let pattern_color = Vector3::new(r, g, b);

    // Mezclar más del patrón para colores más vibrantes
    let base_color = Vector3::new(0.8, 0.8, 0.9);
    let mixed = base_color * 0.2 + pattern_color * 0.8;
    
    Vector3::new(
        mixed.x.max(0.1),  // Mínimo de brillo
        mixed.y.max(0.1),
        mixed.z.max(0.1)
    )
}

// Mercury: CELLULAR NOISE (Worley/Cellular) - Con corona
pub fn cellular_planet_shader(fragment: &Fragment, t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    let cellular = cellular_noise_3d(pos.x as f64 * 6.0, pos.y as f64 * 6.0, pos.z as f64 * 6.0);
    let cellular_pattern = ((cellular + 1.0) * 0.5) as f32;
    
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    // Corona (rim light) - brillo en los bordes
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(2.5) * 0.4;
    
    // Colores cálidos pero moderados
    let heat_boost = 0.25; // Reducido significativamente
    let color_mod = Vector3::new(
        cellular_pattern * 0.5 + heat_boost,
        cellular_pattern * 0.4 + heat_boost * 0.5,
        cellular_pattern * 0.2,
    );
    
    Vector3::new(
        ((base_color.x * 1.2 + color_mod.x) * (0.2 + diff * 0.8) + rim).clamp(0.0, 1.0),
        ((base_color.y * 1.1 + color_mod.y) * (0.2 + diff * 0.8) + rim * 0.8).clamp(0.0, 1.0),
        ((base_color.z * 0.8 + color_mod.z) * (0.2 + diff * 0.8) + rim * 0.6).clamp(0.0, 1.0),
    )
}

// Venus: SIMPLEX NOISE (OpenSimplex) - Con corona
pub fn simplex_planet_shader(fragment: &Fragment, t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    let simplex1 = simplex_noise_3d(pos.x as f64 * 4.0 + t as f64 * 0.2,
                                     pos.y as f64 * 4.0,
                                     pos.z as f64 * 4.0 + t as f64 * 0.2);
    let simplex2 = simplex_noise_3d(pos.x as f64 * 8.0, pos.y as f64 * 8.0, pos.z as f64 * 8.0);
    
    let pattern = ((simplex1 + simplex2) * 0.5 + 1.0) * 0.5;
    
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    let swirl = (pattern * 3.14159 * 2.0).sin() * 0.3;
    
    // Corona (rim light) - brillo en los bordes
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(2.5) * 0.5;
    
    // Atmosfera cálida pero no excesiva
    let heat_glow = 0.2; // Reducido mucho
    Vector3::new(
        ((base_color.x * 1.3 + swirl as f32 + heat_glow) * (0.25 + diff * 0.75) + rim).clamp(0.0, 1.0),
        ((base_color.y * 1.2 + swirl as f32 * 0.8 + heat_glow * 0.7) * (0.25 + diff * 0.75) + rim * 0.8).clamp(0.0, 1.0),
        ((base_color.z * 0.7 + swirl as f32 * 0.5) * (0.25 + diff * 0.75) + rim * 0.6).clamp(0.0, 1.0),
    )
}

// Earth: VORONOI NOISE (Worley con seed diferente) - Con corona
pub fn voronoi_planet_shader(fragment: &Fragment, t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    let voronoi = voronoi_noise_3d(pos.x as f64 * 5.0, pos.y as f64 * 5.0, pos.z as f64 * 5.0);
    let voronoi_pattern = ((voronoi + 1.0) * 0.5) as f32;
    
    let detail = perlin_noise_3d(pos.x as f64 * 10.0, pos.y as f64 * 10.0, pos.z as f64 * 10.0);
    let combined = voronoi_pattern * 0.7 + ((detail + 1.0) * 0.5) as f32 * 0.3;
    
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    // Corona (rim light) - brillo en los bordes
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(2.5) * 0.45;
    
    // Colores cálidos moderados
    let heat_shift = 0.2; // Reducido mucho
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

// Mars: PERLIN NOISE (clásico) - Con corona
pub fn perlin_planet_shader(fragment: &Fragment, t: f32, base_color: Vector3) -> Vector3 {
    let pos = fragment.world_position;
    
    let perlin1 = perlin_noise_3d(pos.x as f64 * 3.0, pos.y as f64 * 3.0, pos.z as f64 * 3.0);
    let perlin2 = perlin_noise_3d(pos.x as f64 * 7.0, pos.y as f64 * 7.0, pos.z as f64 * 7.0);
    let perlin3 = perlin_noise_3d(pos.x as f64 * 15.0, pos.y as f64 * 15.0, pos.z as f64 * 15.0);
    
    let multi_scale = ((perlin1 + 1.0) * 0.5 * 0.5 + (perlin2 + 1.0) * 0.5 * 0.3 + (perlin3 + 1.0) * 0.5 * 0.2) as f32;
    
    let light_dir = Vector3::new(0.5, 1.0, -0.3).normalized();
    let diff = fragment.normal.dot(light_dir).max(0.0);
    
    // Corona (rim light) - brillo en los bordes
    let view_dir = Vector3::new(0.0, 0.0, -1.0).normalized();
    let rim = (1.0 - fragment.normal.dot(view_dir).abs()).powf(2.5) * 0.5;
    
    // Marte cálido pero no excesivo
    let heat_boost = 0.3; // Reducido mucho
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

pub trait Vector3Ext {
    fn normalized(self) -> Self;
    fn dot(self, other: Self) -> f32;
}

impl Vector3Ext for Vector3 {
    fn normalized(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len > 0.0001 { Vector3::new(self.x / len, self.y / len, self.z / len) } else { self }
    }
    fn dot(self, other: Self) -> f32 { self.x * other.x + self.y * other.y + self.z * other.z }
}
