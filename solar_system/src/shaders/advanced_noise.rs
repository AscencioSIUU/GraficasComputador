// Funciones avanzadas de noise para coronas planetarias

use super::common::{perlin_noise_3d, fbm_noise_3d};

/// Worley/Voronoi noise - Para Earth
/// Crea patrones celulares (como células o grietas)
#[inline(always)]
pub fn worley_noise_3d(x: f64, y: f64, z: f64) -> f64 {
    let xi = x.floor();
    let yi = y.floor();
    let zi = z.floor();
    
    let mut min_dist = f64::MAX;
    
    // Buscar en las 27 celdas vecinas (3x3x3)
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let cell_x = xi + dx as f64;
                let cell_y = yi + dy as f64;
                let cell_z = zi + dz as f64;
                
                // Punto característico de la celda (hash simple)
                let hash = ((cell_x * 127.1 + cell_y * 311.7 + cell_z * 74.7).sin() * 43758.5453).fract();
                let hash2 = ((cell_x * 269.5 + cell_y * 183.3 + cell_z * 246.1).sin() * 43758.5453).fract();
                let hash3 = ((cell_x * 419.2 + cell_y * 371.9 + cell_z * 168.2).sin() * 43758.5453).fract();
                
                let point_x = cell_x + hash;
                let point_y = cell_y + hash2;
                let point_z = cell_z + hash3;
                
                // Distancia al punto
                let dx = x - point_x;
                let dy = y - point_y;
                let dz = z - point_z;
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                
                min_dist = min_dist.min(dist);
            }
        }
    }
    
    // Normalizar a rango [0, 1]
    (1.0 - min_dist).clamp(0.0, 1.0)
}

/// Value noise - Para Goliath
/// Similar a Perlin pero con interpolación más simple
#[inline(always)]
pub fn value_noise_3d(x: f64, y: f64, z: f64) -> f64 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    
    let xf = x - x.floor();
    let yf = y - y.floor();
    let zf = z - z.floor();
    
    // Suavizado (smoothstep)
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let w = zf * zf * (3.0 - 2.0 * zf);
    
    // Hash para cada esquina del cubo
    let hash = |x: i32, y: i32, z: i32| -> f64 {
        let n = x.wrapping_mul(127)
            .wrapping_add(y.wrapping_mul(311))
            .wrapping_add(z.wrapping_mul(74));
        ((n as f64 * 43758.5453).sin().fract() * 2.0 - 1.0)
    };
    
    // Interpolación trilineal
    let c000 = hash(xi, yi, zi);
    let c100 = hash(xi + 1, yi, zi);
    let c010 = hash(xi, yi + 1, zi);
    let c110 = hash(xi + 1, yi + 1, zi);
    let c001 = hash(xi, yi, zi + 1);
    let c101 = hash(xi + 1, yi, zi + 1);
    let c011 = hash(xi, yi + 1, zi + 1);
    let c111 = hash(xi + 1, yi + 1, zi + 1);
    
    let c00 = c000 * (1.0 - u) + c100 * u;
    let c01 = c001 * (1.0 - u) + c101 * u;
    let c10 = c010 * (1.0 - u) + c110 * u;
    let c11 = c011 * (1.0 - u) + c111 * u;
    
    let c0 = c00 * (1.0 - v) + c10 * v;
    let c1 = c01 * (1.0 - v) + c11 * v;
    
    c0 * (1.0 - w) + c1 * w
}

/// FBM (Fractional Brownian Motion) mejorado - Para Mars
/// Ya existe en common.rs pero esta versión es más configurable
#[inline(always)]
pub fn fbm_enhanced(x: f64, y: f64, z: f64, octaves: u32, lacunarity: f64, gain: f64) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    
    for _ in 0..octaves.min(4) {
        value += perlin_noise_3d(x * frequency, y * frequency, z * frequency) as f64 * amplitude;
        frequency *= lacunarity;
        amplitude *= gain;
    }
    
    value
}

/// Domain Warping - Para Mercury
/// Distorsiona el espacio antes de aplicar noise
#[inline(always)]
pub fn domain_warp_3d(x: f64, y: f64, z: f64, time: f64) -> f64 {
    // Primera capa de noise para distorsionar el dominio
    let warp_x = perlin_noise_3d(x * 2.0 + time, y * 2.0, z * 2.0) as f64;
    let warp_y = perlin_noise_3d(x * 2.0, y * 2.0 + time, z * 2.0) as f64;
    let warp_z = perlin_noise_3d(x * 2.0, y * 2.0, z * 2.0 + time) as f64;
    
    // Aplicar distorsión
    let warped_x = x + warp_x * 0.5;
    let warped_y = y + warp_y * 0.5;
    let warped_z = z + warp_z * 0.5;
    
    // Segunda capa de noise en el espacio distorsionado
    perlin_noise_3d(warped_x * 3.0, warped_y * 3.0, warped_z * 3.0) as f64
}

/// Ridged Multifractal - Para Venus
/// Crea patrones de crestas/montañas invertidas
#[inline(always)]
pub fn ridged_multifractal(x: f64, y: f64, z: f64, octaves: u32) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut weight = 1.0;
    
    for _ in 0..octaves.min(4) {
        // Obtener noise y crear cresta (invertir y hacer absoluto)
        let mut signal = perlin_noise_3d(x * frequency, y * frequency, z * frequency) as f64;
        signal = signal.abs();
        signal = 1.0 - signal; // Invertir para crear crestas
        
        // Elevar al cuadrado para acentuar las crestas
        signal = signal * signal;
        
        // Peso basado en la octava anterior
        signal *= weight;
        weight = (signal * 2.0).clamp(0.0, 1.0);
        
        value += signal * amplitude;
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    
    value
}
