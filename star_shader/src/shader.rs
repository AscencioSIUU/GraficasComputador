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
    // Usamos múltiples capas de ruido para crear turbulencia compleja EXAGERADA
    let turbulence_scale = 8.0; // Aumentado de 3.0 para más detalle
    let turbulence_raw = fbm_noise_3d(
        pos.x as f64 * turbulence_scale + time * 0.4,  // Velocidad 4x más rápida
        pos.y as f64 * turbulence_scale + time * 0.5,
        pos.z as f64 * turbulence_scale + time * 0.45,
        8 // Más octavas para ruido más caótico
    );
    // Normalizar y exagerar el contraste
    let turbulence = ((turbulence_raw + 1.0) * 0.5).powf(0.6); // Exponente para más contraste
    
    // ========== 2. MANCHAS SOLARES (SUNSPOTS) ==========
    // Zonas MUY oscuras y grandes que se mueven rápido
    let spot_noise = perlin_noise_3d(
        pos.x as f64 * 4.0 + time * 0.2,  // Frecuencia y velocidad aumentadas
        pos.y as f64 * 4.0,
        pos.z as f64 * 4.0 + time * 0.2
    );
    let sunspots = if spot_noise < 0.0 {  // Threshold más agresivo
        (spot_noise.abs() * 3.0).min(0.8)  // Mucho más oscuro
    } else { 
        0.0 
    };
    
    // ========== 3. PROMINENCIAS Y ERUPCIONES SOLARES ==========
    // Burbujas VIOLENTAS de actividad intensa
    let flare_noise_raw = turbulence_noise_3d(
        pos.x as f64 * 12.0 + (time * 4.0).sin() * 2.0,  // Mucho más rápido y agresivo
        pos.y as f64 * 12.0 + time * 1.2,
        pos.z as f64 * 12.0 + (time * 4.0).cos() * 2.0
    );
    // Turbulence muy exagerado
    let flare_noise = (flare_noise_raw.abs() * 1.5).min(2.0);
    
    // Pulsaciones INTENSAS (ciclo solar acelerado)
    let pulse = ((time * 3.0).sin() * 0.5 + 0.5) as f32;  // 2x más rápido
    let flares = if flare_noise > 0.3 {  // Threshold más bajo para más erupciones
        (flare_noise - 0.3) * 4.0 * pulse  // Mucho más intenso
    } else { 
        0.0 
    };
    
    // ========== 4. ACTIVIDAD SUPERFICIAL ANIMADA ==========
    // Células de convección VIOLENTAS (granulación extrema)
    let granulation_raw = fbm_noise_3d(
        pos.x as f64 * 30.0 + time * 1.5,  // Mucho más fino y rápido
        pos.y as f64 * 30.0 + time * 1.8,
        pos.z as f64 * 30.0 + time * 1.65,
        6  // Más octavas
    );
    let granulation = ((granulation_raw + 1.0) * 0.5).powf(0.5) * 0.4;  // Mucho más visible
    
    // ========== 5. CALCULAR INTENSIDAD COMBINADA ==========
    let mut intensity = 0.7; // Base
    intensity += turbulence * 0.6; // MUCHO más turbulencia
    intensity -= sunspots * 0.7; // Manchas MUY oscuras
    intensity += flares * 1.2; // Erupciones EXTREMAS
    intensity += granulation * 0.8; // Granulación muy visible
    
    // Aplicar intensidad global del uniform
    intensity *= uniforms.intensity;
    intensity = intensity.clamp(0.1, 2.5); // Rango más amplio para contraste extremo
    
    // ========== 6. GRADIENTE DE TEMPERATURA A COLOR ==========
    // Mapear intensidad a color realista de estrella
    // Rojo oscuro -> Naranja -> Amarillo -> Blanco -> Azul (estrellas muy calientes)
    let temp_factor = uniforms.temperature;
    let color = temperature_to_color(intensity, temp_factor);
    
    // ========== 7. EMISIÓN DE LUZ VARIABLE ==========
    // Las zonas más intensas emiten MUCHA más luz (picos dramáticos)
    let emission = (intensity * intensity * 2.0).min(3.0); // Cuadrático para picos extremos
    
    // ========== 8. ILUMINACIÓN SUAVE (opcional) ==========
    // Las estrellas son auto-luminosas, pero añadimos forma sutil
    let light_dir = Vector3::new(0.5, 1.0, 0.3).normalized();
    let light_factor = (normal.dot(light_dir) * 0.3 + 0.7).max(0.4);
    
    // ========== 9. COLOR FINAL ==========
    Vector3::new(
        (color.x * light_factor + emission * 0.8).clamp(0.0, 1.0),  // Mucha más emisión
        (color.y * light_factor + emission * 0.7).clamp(0.0, 1.0),
        (color.z * light_factor + emission * 0.6).clamp(0.0, 1.0),
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

/// Desplaza vértices para crear efecto de corona solar y flares EXTREMOS
pub fn vertex_displacement(position: Vector3, time: f32) -> Vector3 {
    let t = time as f64;
    
    // Corona solar VIOLENTA (expansión radial extrema)
    let corona_noise = perlin_noise_3d(
        position.x as f64 * 8.0 + t * 0.8,  // Más rápido y detallado
        position.y as f64 * 8.0 + t * 1.0,
        position.z as f64 * 8.0 + t * 0.9
    );
    
    // Prominencias GIGANTES (extensiones direccionales masivas)
    let prominence = fbm_noise_3d(
        position.x as f64 * 10.0,
        position.y as f64 * 10.0 + t * 1.5,  // Mucho más rápido
        position.z as f64 * 10.0,
        5  // Más octavas para más caos
    );
    
    // Desplazamiento radial EXTREMO
    let displacement = (corona_noise.abs() * 0.25 + prominence.abs() * 0.35) as f32;  // 3x más grande
    
    // Normalizar posición para obtener dirección radial
    let len = (position.x * position.x + position.y * position.y + position.z * position.z).sqrt();
    let direction = if len > 0.001 {
        Vector3::new(position.x / len, position.y / len, position.z / len)
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    };
    
    // Aplicar desplazamiento MASIVO
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
