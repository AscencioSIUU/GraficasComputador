use raylib::prelude::*;

/// Propiedades físicas de un material
#[derive(Clone, Debug)]
pub struct MaterialProperties {
    /// Color base (usado cuando no hay textura o se mezcla con ella)
    pub albedo: Color,
    
    /// Factor especular (0.0 = mate, 1.0 = espejo perfecto)
    pub specular: f32,
    
    /// Transparencia (0.0 = opaco, 1.0 = completamente transparente)
    pub transparency: f32,
    
    /// Reflectividad (0.0 = no refleja, 1.0 = espejo perfecto)
    pub reflectivity: f32,
    
    /// Índice de refracción (1.0 = aire, 1.33 = agua, 1.5 = vidrio)
    pub refractive_index: f32,
    
    /// Es emisivo (emite luz)
    pub emissive: bool,
    
    /// Color y fuerza de emisión
    pub emission_color: Color,
    pub emission_strength: f32,
}

impl MaterialProperties {
    /// Material grass block (césped)
    pub fn grass() -> Self {
        Self {
            albedo: Color::new(100, 180, 70, 255),
            specular: 0.1,
            transparency: 0.0,
            reflectivity: 0.0,
            refractive_index: 1.0,
            emissive: false,
            emission_color: Color::BLACK,
            emission_strength: 0.0,
        }
    }

    /// Material stone (piedra)
    pub fn stone() -> Self {
        Self {
            albedo: Color::new(128, 128, 128, 255),
            specular: 0.3,
            transparency: 0.0,
            reflectivity: 0.05,
            refractive_index: 1.0,
            emissive: false,
            emission_color: Color::BLACK,
            emission_strength: 0.0,
        }
    }

    /// Material water (agua) - transparente, refractivo, reflectivo
    pub fn water() -> Self {
        Self {
            albedo: Color::new(50, 100, 200, 180),
            specular: 0.9,
            transparency: 0.7,
            reflectivity: 0.4,
            refractive_index: 1.33,
            emissive: false,
            emission_color: Color::BLACK,
            emission_strength: 0.0,
        }
    }

    /// Material glass (vidrio) - muy transparente, altamente refractivo
    pub fn glass() -> Self {
        Self {
            albedo: Color::new(200, 230, 255, 100),
            specular: 0.95,
            transparency: 0.9,
            reflectivity: 0.2,
            refractive_index: 1.5,
            emissive: false,
            emission_color: Color::BLACK,
            emission_strength: 0.0,
        }
    }

    /// Material torch (antorcha) - emisivo
    pub fn torch() -> Self {
        Self {
            albedo: Color::new(255, 180, 60, 255),
            specular: 0.0,
            transparency: 0.0,
            reflectivity: 0.0,
            refractive_index: 1.0,
            emissive: true,
            emission_color: Color::new(255, 180, 60, 255),
            emission_strength: 3.0,
        }
    }

    /// Material portal (portal) - emisivo con efecto especial
    pub fn portal() -> Self {
        Self {
            albedo: Color::new(150, 50, 255, 255),
            specular: 0.5,
            transparency: 0.3,
            reflectivity: 0.3,
            refractive_index: 1.2,
            emissive: true,
            emission_color: Color::new(150, 50, 255, 255),
            emission_strength: 2.0,
        }
    }

    /// Material dirt (tierra)
    pub fn dirt() -> Self {
        Self {
            albedo: Color::new(134, 96, 67, 255),
            specular: 0.05,
            transparency: 0.0,
            reflectivity: 0.0,
            refractive_index: 1.0,
            emissive: false,
            emission_color: Color::BLACK,
            emission_strength: 0.0,
        }
    }

    /// Material sand (arena)
    pub fn sand() -> Self {
        Self {
            albedo: Color::new(237, 201, 175, 255),
            specular: 0.1,
            transparency: 0.0,
            reflectivity: 0.0,
            refractive_index: 1.0,
            emissive: false,
            emission_color: Color::BLACK,
            emission_strength: 0.0,
        }
    }
}

/// Tipo de material para facilitar la creación
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialType {
    Grass,
    Stone,
    Water,
    Glass,
    Torch,
    Portal,
    Dirt,
    Sand,
}

impl MaterialType {
    pub fn properties(&self) -> MaterialProperties {
        match self {
            MaterialType::Grass => MaterialProperties::grass(),
            MaterialType::Stone => MaterialProperties::stone(),
            MaterialType::Water => MaterialProperties::water(),
            MaterialType::Glass => MaterialProperties::glass(),
            MaterialType::Torch => MaterialProperties::torch(),
            MaterialType::Portal => MaterialProperties::portal(),
            MaterialType::Dirt => MaterialProperties::dirt(),
            MaterialType::Sand => MaterialProperties::sand(),
        }
    }
}
