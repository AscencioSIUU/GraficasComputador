use raylib::math::Vector3;
use noise::{NoiseFn, Perlin, Fbm, Turbulence, MultiFractal};

pub struct Fragment {
    pub world_position: Vector3,
    pub normal: Vector3,
}

pub struct Uniforms {
    pub time: f32,
    pub intensity: f32,  // Control de intensidad global
    pub temperature: f32, // Control de temperatura de la estrella
}

impl Uniforms {
    pub fn new(time: f32, intensity: f32, temperature: f32) -> Self {
        Uniforms { time, intensity, temperature }
    }
}

// ========== FUNCIONES DE RUIDO USANDO LA LIBRERÍA ==========

/// Perlin noise 3D usando la librería noise
fn perlin_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    let perlin = Perlin::new(42);
    perlin.get([x, y, z]) as f32
}

/// Fractal Brownian Motion usando la librería noise
fn fbm_noise_3d(x: f64, y: f64, z: f64, octaves: usize) -> f32 {
    let fbm = Fbm::<Perlin>::new(42)
        .set_octaves(octaves);
    fbm.get([x, y, z]) as f32
}

/// Turbulence (ruido turbulento) usando la librería noise
fn turbulence_noise_3d(x: f64, y: f64, z: f64) -> f32 {
    let turbulence = Turbulence::<_, Perlin>::new(Perlin::new(42));
    turbulence.get([x, y, z]) as f32
}

// ========== SHADER DE ESTRELLA AVANZADO ==========

/// Shader principal de la estrella con todos los efectos
pub fn star_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let normal = fragment.normal;
    let time = uniforms.time as f64;
    
    // ========== 1. TURBULENCIA SOLAR BASE ==========
    // Usamos múltiples capas de ruido para crear turbulencia compleja
    let turbulence_scale = 3.0;
    let turbulence_raw = fbm_noise_3d(
        pos.x as f64 * turbulence_scale + time * 0.1,
        pos.y as f64 * turbulence_scale + time * 0.15,
        pos.z as f64 * turbulence_scale + time * 0.12,
        6 // 6 octavas para detalle fino
    );
    // Normalizar de [-1, 1] a [0, 1]
    let turbulence = (turbulence_raw + 1.0) * 0.5;
    
    // ========== 2. MANCHAS SOLARES (SUNSPOTS) ==========
    // Zonas más oscuras que se mueven lentamente
    let spot_noise = perlin_noise_3d(
        pos.x as f64 * 2.0 + time * 0.05,
        pos.y as f64 * 2.0,
        pos.z as f64 * 2.0 + time * 0.05
    );
    let sunspots = if spot_noise < -0.2 { 
        ((spot_noise + 0.2).abs() * 1.5).min(0.3)
    } else { 
        0.0 
    };
    
    // ========== 3. PROMINENCIAS Y ERUPCIONES SOLARES ==========
    // Burbujas de actividad intensa que se expanden y contraen
    let flare_noise_raw = turbulence_noise_3d(
        pos.x as f64 * 5.0 + (time * 2.0).sin() * 0.5,
        pos.y as f64 * 5.0 + time * 0.3,
        pos.z as f64 * 5.0 + (time * 2.0).cos() * 0.5
    );
    // Turbulence devuelve valores positivos, normalizar
    let flare_noise = (flare_noise_raw.abs() * 0.5).min(1.0);
    
    // Pulsaciones periódicas (ciclo solar)
    let pulse = ((time * 1.5).sin() * 0.5 + 0.5) as f32;
    let flares = if flare_noise > 0.6 { 
        (flare_noise - 0.6) * 2.0 * pulse
    } else { 
        0.0 
    };
    
    // ========== 4. ACTIVIDAD SUPERFICIAL ANIMADA ==========
    // Células de convección (granulación)
    let granulation_raw = fbm_noise_3d(
        pos.x as f64 * 15.0 + time * 0.5,
        pos.y as f64 * 15.0 + time * 0.6,
        pos.z as f64 * 15.0 + time * 0.55,
        4
    );
    let granulation = ((granulation_raw + 1.0) * 0.5) * 0.15;
    
    // ========== 5. CALCULAR INTENSIDAD COMBINADA ==========
    let mut intensity = 0.8; // Base más alta
    intensity += turbulence * 0.25; // Turbulencia general
    intensity -= sunspots * 0.35; // Manchas oscuras
    intensity += flares * 0.4; // Erupciones brillantes
    intensity += granulation; // Granulación fina
    
    // Aplicar intensidad global del uniform
    intensity *= uniforms.intensity;
    intensity = intensity.clamp(0.3, 1.8); // Nunca completamente negro
    
    // ========== 6. GRADIENTE DE TEMPERATURA A COLOR ==========
    // Mapear intensidad a color realista de estrella
    // Rojo oscuro -> Naranja -> Amarillo -> Blanco -> Azul (estrellas muy calientes)
    let temp_factor = uniforms.temperature;
    let color = temperature_to_color(intensity, temp_factor);
    
    // ========== 7. EMISIÓN DE LUZ VARIABLE ==========
    // Las zonas más intensas emiten más luz
    let emission = intensity * 1.2; // Aumentado para más brillo
    
    // ========== 8. ILUMINACIÓN SUAVE (opcional) ==========
    // Las estrellas son auto-luminosas, pero añadimos forma sutil
    let light_dir = Vector3::new(0.5, 1.0, 0.3).normalized();
    let light_factor = (normal.dot(light_dir) * 0.2 + 0.8).max(0.5);
    
    // ========== 9. COLOR FINAL ==========
    Vector3::new(
        (color.x * light_factor + emission * 0.5).clamp(0.0, 1.0),
        (color.y * light_factor + emission * 0.4).clamp(0.0, 1.0),
        (color.z * light_factor + emission * 0.3).clamp(0.0, 1.0),
    )
}

/// Mapea intensidad y temperatura a color realista de estrella
/// Basado en diagrama Hertzsprung-Russell (clasificación estelar)
fn temperature_to_color(intensity: f32, temp_factor: f32) -> Vector3 {
    // temp_factor: 0.0 = estrella fría (roja), 1.0 = estrella caliente (azul)
    
    if temp_factor < 0.3 {
        // Estrella roja/naranja (tipo M, K)
        let t = (intensity * 1.2).clamp(0.0, 1.0);
        Vector3::new(
            1.0,                     // Rojo máximo
            0.3 + t * 0.5,           // Verde bajo-medio
            0.1 + t * 0.2,           // Azul muy bajo
        )
    } else if temp_factor < 0.6 {
        // Estrella amarilla (tipo G, F - como nuestro Sol)
        let t = (intensity * 1.0).clamp(0.0, 1.0);
        Vector3::new(
            1.0,                     // Rojo máximo
            0.9 + t * 0.1,           // Verde alto
            0.4 + t * 0.4,           // Azul medio
        )
    } else {
        // Estrella blanca/azul (tipo A, B, O)
        let t = intensity.clamp(0.0, 1.0);
        Vector3::new(
            0.8 + t * 0.2,           // Rojo medio-alto
            0.9 + t * 0.1,           // Verde alto
            1.0,                     // Azul máximo
        )
    }
}

// ========== VERTEX SHADER (DISTORSIÓN) ==========

/// Desplaza vértices para crear efecto de corona solar y flares
pub fn vertex_displacement(position: Vector3, time: f32) -> Vector3 {
    let t = time as f64;
    
    // Corona solar (expansión radial)
    let corona_noise = perlin_noise_3d(
        position.x as f64 * 4.0 + t * 0.2,
        position.y as f64 * 4.0 + t * 0.25,
        position.z as f64 * 4.0 + t * 0.22
    );
    
    // Prominencias (extensiones direccionales)
    let prominence = fbm_noise_3d(
        position.x as f64 * 6.0,
        position.y as f64 * 6.0 + t * 0.5,
        position.z as f64 * 6.0,
        3
    );
    
    // Calcular desplazamiento radial
    let displacement = (corona_noise * 0.08 + prominence.abs() * 0.12) as f32;
    
    // Normalizar posición para obtener dirección radial
    let len = (position.x * position.x + position.y * position.y + position.z * position.z).sqrt();
    let direction = if len > 0.001 {
        Vector3::new(position.x / len, position.y / len, position.z / len)
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    };
    
    // Aplicar desplazamiento
    Vector3::new(
        position.x + direction.x * displacement,
        position.y + direction.y * displacement,
        position.z + direction.z * displacement,
    )
}

// ========== UTILIDADES ==========

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
