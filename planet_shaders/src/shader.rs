use raylib::math::Vector3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanetType {
    Rocky,
    GasGiant,
    Ice,
    Lava,
    Earth,
    Moon,
    BlackHole,
}

pub struct Fragment {
    pub world_position: Vector3,
    pub normal: Vector3,
}

pub struct Uniforms {
    pub time: f32,
    pub planet_type: PlanetType,
}

// Trait helper para Vector3
trait Vector3Ext {
    fn normalized(self) -> Self;
    fn dot(self, other: Self) -> f32;
    fn length(self) -> f32;
}

impl Vector3Ext for Vector3 {
    fn normalized(self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Vector3::new(self.x / len, self.y / len, self.z / len)
        } else {
            self
        }
    }
    
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

impl Uniforms {
    pub fn new(time: f32, planet_type: PlanetType) -> Self {
        Uniforms { time, planet_type }
    }

    pub fn fragment_shader(&self, world_pos: Vector3, normal: Vector3) -> Vector3 {
        match self.planet_type {
            PlanetType::Rocky => Self::rocky_shader(world_pos, normal, self.time),
            PlanetType::GasGiant => Self::gas_giant_shader(world_pos, normal, self.time),
            PlanetType::Ice => Self::ice_shader(world_pos, normal, self.time),
            PlanetType::Lava => Self::lava_shader(world_pos, normal, self.time),
            PlanetType::Earth => Self::earth_shader(world_pos, normal, self.time),
            PlanetType::Moon => Self::moon_shader(world_pos, normal, self.time),
            PlanetType::BlackHole => Vector3::new(0.0, 0.0, 0.0), // No usado (GPU shader)
        }
    }

    // ========== FUNCIONES DE RUIDO MEJORADAS ==========

    fn hash(p: Vector3) -> f32 {
        let p = Vector3::new(
            p.x.sin() * 43758.5453,
            p.y.sin() * 22578.1459,
            p.z.sin() * 19642.3490,
        );
        (p.x * 12.9898 + p.y * 78.233 + p.z * 45.164).sin().fract()
    }

    fn noise3d(p: Vector3) -> f32 {
        let i = Vector3::new(p.x.floor(), p.y.floor(), p.z.floor());
        let f = Vector3::new(p.x.fract(), p.y.fract(), p.z.fract());
        
        let u = Vector3::new(
            f.x * f.x * (3.0 - 2.0 * f.x),
            f.y * f.y * (3.0 - 2.0 * f.y),
            f.z * f.z * (3.0 - 2.0 * f.z),
        );

        let mut res = 0.0;
        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    let corner = Vector3::new(i.x + x as f32, i.y + y as f32, i.z + z as f32);
                    let h = Self::hash(corner);
                    let wx = if x == 0 { 1.0 - u.x } else { u.x };
                    let wy = if y == 0 { 1.0 - u.y } else { u.y };
                    let wz = if z == 0 { 1.0 - u.z } else { u.z };
                    res += wx * wy * wz * h;
                }
            }
        }
        res
    }

    fn fbm(p: Vector3, octaves: i32) -> f32 {
        let mut value = 0.0;
        let mut amplitude = 0.5;
        let mut frequency = 1.0;
        
        for _ in 0..octaves {
            value += amplitude * Self::noise3d(p * frequency);
            frequency *= 2.0;
            amplitude *= 0.5;
        }
        value
    }

    fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    fn mix(a: f32, b: f32, t: f32) -> f32 {
        a * (1.0 - t) + b * t
    }

    fn lerp_color(a: Vector3, b: Vector3, t: f32) -> Vector3 {
        Vector3::new(
            Self::mix(a.x, b.x, t),
            Self::mix(a.y, b.y, t),
            Self::mix(a.z, b.z, t),
        )
    }

    // Blinn-Phong lighting mejorado
    fn calculate_lighting(normal: Vector3, view_dir: Vector3, light_dir: Vector3, 
                         diffuse_color: Vector3, specular: f32, shininess: f32, rim_power: f32) -> Vector3 {
        let n = normal.normalized();
        let l = light_dir.normalized();
        let v = view_dir.normalized();

        // Diffuse (Lambert)
        let ndotl = n.dot(l).max(0.0);
        let diffuse = diffuse_color * ndotl;

        // Specular (Blinn-Phong)
        let h = Vector3::new(l.x + v.x, l.y + v.y, l.z + v.z).normalized();
        let ndoth = n.dot(h).max(0.0);
        let spec = ndoth.powf(shininess) * specular;

        // Rim lighting (Fresnel)
        let rim = (1.0 - n.dot(v).max(0.0)).powf(rim_power) * 0.3;

        // Ambient
        let ambient = diffuse_color * 0.15;

        Vector3::new(
            (ambient.x + diffuse.x + spec + rim).min(1.0),
            (ambient.y + diffuse.y + spec + rim).min(1.0),
            (ambient.z + diffuse.z + spec * 0.8 + rim).min(1.0),
        )
    }

    // ========== SHADERS DE PLANETAS REESCRITOS ==========

    fn rocky_shader(world_pos: Vector3, normal: Vector3, _time: f32) -> Vector3 {
        // Múltiples capas de ruido para textura rica
        let base_noise = Self::fbm(world_pos * 2.5, 4);
        let detail_noise = Self::fbm(world_pos * 8.0, 3) * 0.5;
        let crater_noise = Self::fbm(world_pos * 12.0, 2);
        
        // Definir regiones de color
        let rock_dark = Vector3::new(0.35, 0.28, 0.22);
        let rock_mid = Vector3::new(0.55, 0.45, 0.35);
        let rock_light = Vector3::new(0.75, 0.65, 0.55);
        let crater_dark = Vector3::new(0.18, 0.15, 0.13);
        
        // Mezclar colores base
        let base_mix = Self::smoothstep(0.3, 0.7, base_noise);
        let mut color = Self::lerp_color(rock_dark, rock_mid, base_mix);
        color = Self::lerp_color(color, rock_light, Self::smoothstep(0.65, 0.85, detail_noise));
        
        // Aplicar cráteres
        let crater_mask = Self::smoothstep(0.68, 0.75, crater_noise);
        color = Self::lerp_color(color, crater_dark, crater_mask * 0.7);
        
        // Lighting
        let view_dir = Vector3::new(-world_pos.x, -world_pos.y, -world_pos.z);
        let light_dir = Vector3::new(1.0, 1.0, -0.5);
        
        Self::calculate_lighting(normal, view_dir, light_dir, color, 0.25, 32.0, 3.5)
    }

    fn ice_shader(world_pos: Vector3, normal: Vector3, time: f32) -> Vector3 {
        // Cristales de hielo con animación sutil
        let crystals = Self::fbm(world_pos * 6.0 + Vector3::new(time * 0.05, 0.0, 0.0), 4);
        let cracks = Self::fbm(world_pos * 15.0, 2);
        
        // Colores de hielo: azul brillante a blanco
        let ice_deep = Vector3::new(0.65, 0.82, 0.95);
        let ice_bright = Vector3::new(0.92, 0.96, 1.0);
        let ice_crystal = Vector3::new(0.98, 0.99, 1.0);
        
        let crystal_mix = Self::smoothstep(0.45, 0.65, crystals);
        let mut color = Self::lerp_color(ice_deep, ice_bright, crystal_mix);
        
        // Añadir brillo cristalino
        let crystal_highlight = Self::smoothstep(0.75, 0.85, cracks);
        color = Self::lerp_color(color, ice_crystal, crystal_highlight * 0.6);
        
        let view_dir = Vector3::new(-world_pos.x, -world_pos.y, -world_pos.z);
        let light_dir = Vector3::new(1.0, 1.0, -0.5);
        
        // Hielo tiene alta especularidad
        Self::calculate_lighting(normal, view_dir, light_dir, color, 0.75, 180.0, 2.8)
    }

    fn lava_shader(world_pos: Vector3, normal: Vector3, time: f32) -> Vector3 {
        // Flujo de lava animado
        let flow1 = Self::fbm(world_pos * 3.0 + Vector3::new(time * 0.15, 0.0, time * 0.1), 4);
        let flow2 = Self::fbm(world_pos * 7.0 + Vector3::new(-time * 0.08, time * 0.05, 0.0), 3);
        let cracks = Self::fbm(world_pos * 12.0, 2);
        
        // Colores de lava: negro → rojo → naranja → amarillo → blanco
        let rock_cold = Vector3::new(0.12, 0.08, 0.06);
        let lava_red = Vector3::new(0.95, 0.25, 0.08);
        let lava_orange = Vector3::new(1.0, 0.55, 0.15);
        let lava_yellow = Vector3::new(1.0, 0.9, 0.35);
        
        // Determinar temperatura (flujo de lava)
        let flow_combined = (flow1 * 0.6 + flow2 * 0.4).clamp(0.0, 1.0);
        let heat = Self::smoothstep(0.35, 0.75, flow_combined);
        
        // Mezclar según temperatura
        let mut color = if heat > 0.7 {
            Self::lerp_color(lava_orange, lava_yellow, (heat - 0.7) * 3.3)
        } else if heat > 0.3 {
            Self::lerp_color(lava_red, lava_orange, (heat - 0.3) * 2.5)
        } else {
            Self::lerp_color(rock_cold, lava_red, heat * 3.3)
        };
        
        // Grietas incandescentes
        let crack_glow = Self::smoothstep(0.7, 0.85, cracks) * heat;
        color = Self::lerp_color(color, lava_yellow, crack_glow * 0.5);
        
        // Pulsación de brillo
        let pulse = (time * 1.5).sin() * 0.08 + 0.92;
        let emission = heat * pulse * 0.4;
        
        let view_dir = Vector3::new(-world_pos.x, -world_pos.y, -world_pos.z);
        let light_dir = Vector3::new(1.0, 1.0, -0.5);
        
        let lit = Self::calculate_lighting(normal, view_dir, light_dir, color, 0.45, 16.0, 2.0);
        
        // Añadir emisión
        Vector3::new(
            (lit.x + emission).min(1.0),
            (lit.y + emission * 0.7).min(1.0),
            (lit.z + emission * 0.3).min(1.0),
        )
    }

    fn gas_giant_shader(world_pos: Vector3, normal: Vector3, time: f32) -> Vector3 {
        // Bandas atmosféricas horizontales
        let latitude = world_pos.y;
        let bands = (latitude * 10.0).sin() * 0.5 + 0.5;
        
        // Turbulencias y vórtices
        let turb = Self::fbm(world_pos * 4.0 + Vector3::new(time * 0.08, 0.0, -time * 0.05), 5);
        let vortex = Self::fbm(world_pos * 8.0 + Vector3::new((time * 0.3).sin(), 0.0, (time * 0.3).cos()), 3);
        
        // Colores de atmósfera
        let atmos_light = Vector3::new(0.92, 0.82, 0.68);
        let atmos_mid = Vector3::new(0.78, 0.62, 0.45);
        let atmos_dark = Vector3::new(0.58, 0.42, 0.28);
        let storm_red = Vector3::new(0.85, 0.45, 0.32);
        
        // Mezclar bandas
        let band_mix = Self::smoothstep(0.3, 0.7, bands + turb * 0.3);
        let mut color = if band_mix > 0.6 {
            Self::lerp_color(atmos_mid, atmos_light, (band_mix - 0.6) * 2.5)
        } else {
            Self::lerp_color(atmos_dark, atmos_mid, band_mix * 1.67)
        };
        
        // Tormentas (manchas rojas)
        let storm_mask = Self::smoothstep(0.75, 0.88, vortex);
        color = Self::lerp_color(color, storm_red, storm_mask * 0.65);
        
        let view_dir = Vector3::new(-world_pos.x, -world_pos.y, -world_pos.z);
        let light_dir = Vector3::new(1.0, 1.0, -0.5);
        
        Self::calculate_lighting(normal, view_dir, light_dir, color, 0.18, 64.0, 2.5)
    }

    fn earth_shader(world_pos: Vector3, normal: Vector3, _time: f32) -> Vector3 {
        let continents = Self::fbm(world_pos * 2.2, 5);
        
        let ocean = Vector3::new(0.05, 0.15, 0.35);
        let land_green = Vector3::new(0.25, 0.55, 0.25);
        let land_desert = Vector3::new(0.82, 0.72, 0.52);
        
        let is_land = continents > 0.48;
        let land_type = Self::fbm(world_pos * 5.0, 3);
        
        let color = if is_land {
            let desert_mix = Self::smoothstep(0.55, 0.75, land_type);
            Self::lerp_color(land_green, land_desert, desert_mix)
        } else {
            ocean
        };
        
        let view_dir = Vector3::new(-world_pos.x, -world_pos.y, -world_pos.z);
        let light_dir = Vector3::new(1.0, 1.0, -0.5);
        
        let spec = if is_land { 0.15 } else { 0.55 };
        let shine = if is_land { 24.0 } else { 80.0 };
        
        Self::calculate_lighting(normal, view_dir, light_dir, color, spec, shine, 2.2)
    }

    fn moon_shader(world_pos: Vector3, normal: Vector3, _time: f32) -> Vector3 {
        let surface = Self::fbm(world_pos * 5.0, 5);
        let craters = Self::fbm(world_pos * 10.0, 3);
        
        let gray_light = 0.45 + surface * 0.18;
        let crater_depth = Self::smoothstep(0.7, 0.82, craters);
        
        let gray = (gray_light - crater_depth * 0.25).clamp(0.15, 0.7);
        let color = Vector3::new(gray, gray * 0.98, gray * 0.95);
        
        let view_dir = Vector3::new(-world_pos.x, -world_pos.y, -world_pos.z);
        let light_dir = Vector3::new(1.0, 1.0, -0.5);
        
        Self::calculate_lighting(normal, view_dir, light_dir, color, 0.08, 12.0, 1.8)
    }
}
