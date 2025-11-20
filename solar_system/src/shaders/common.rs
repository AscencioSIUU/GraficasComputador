// Tipos y funciones comunes para todos los shaders

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

// ==========================================
// OPTIMIZACIÓN: Funciones de ruido optimizadas
// Reutiliza instancias y limita octavas
// ==========================================

#[inline(always)]
pub fn perlin_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    // Seed fijo para consistencia, sin recrear cada vez
    thread_local! {
        static PERLIN: Perlin = Perlin::new(42);
    }
    PERLIN.with(|p| p.get([x, y, z]) as f32)
}

#[inline(always)]
pub fn fbm_noise_3d(x: f64, y: f64, z: f64, octaves: usize) -> f32 {
    // Limita octavas máximas a 3 para performance
    let oct = octaves.min(3);
    thread_local! {
        static FBM2: Fbm<Perlin> = Fbm::<Perlin>::new(42).set_octaves(2);
        static FBM3: Fbm<Perlin> = Fbm::<Perlin>::new(42).set_octaves(3);
    }
    match oct {
        1 | 2 => FBM2.with(|f| f.get([x, y, z]) as f32),
        _ => FBM3.with(|f| f.get([x, y, z]) as f32),
    }
}

#[inline(always)]
pub fn simplex_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    thread_local! {
        static SIMPLEX: OpenSimplex = OpenSimplex::new(42);
    }
    SIMPLEX.with(|s| s.get([x, y, z]) as f32)
}

#[inline(always)]
pub fn cellular_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    thread_local! {
        static WORLEY: Worley = Worley::new(42);
    }
    WORLEY.with(|w| w.get([x, y, z]) as f32)
}

#[inline(always)]
pub fn voronoi_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    thread_local! {
        static WORLEY2: Worley = Worley::new(123);
    }
    WORLEY2.with(|w| w.get([x, y, z]) as f32)
}

#[inline(always)]
pub fn fbm(x: f32, y: f32, z: f32, octaves: usize) -> f32 {
    fbm_noise_3d(x as f64, y as f64, z as f64, octaves)
}

// Trait para operaciones de Vector3
pub trait Vector3Ext {
    fn normalized(self) -> Self;
    fn dot(self, other: Self) -> f32;
}

impl Vector3Ext for Vector3 {
    fn normalized(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len > 0.0001 { 
            Vector3::new(self.x / len, self.y / len, self.z / len) 
        } else { 
            self 
        }
    }
    
    fn dot(self, other: Self) -> f32 { 
        self.x * other.x + self.y * other.y + self.z * other.z 
    }
}
