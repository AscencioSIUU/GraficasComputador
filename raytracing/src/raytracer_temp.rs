//! Construye la escena de bloques y ejecuta el trazador de rayos en CPU.

use std::collections::HashSet;
use std::thread;

use crate::lighting::{Skybox, Tex, reflect, refract, sample_skybox, sky, specular_phong, to_rgba};
use crate::camera::Camera;
use crate::solid_block::SolidBlock;
use crate::textured_block::TexturedBlock;
use crate::math::Vec3;
use crate::ray::Ray;
use crate::materials::Intersectable;

type DynObject<'a> = Box<dyn Intersectable + Send + Sync + 'a>;

/// Identifica qué diorama (Overworld o Nether) se va a renderizar.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum WorldKind {
    Overworld,
    Nether,
}

impl WorldKind {
    pub fn toggle(self) -> Self {
        match self {
            WorldKind::Overworld => WorldKind::Nether,
            WorldKind::Nether => WorldKind::Overworld,
        }
    }
}

/// Datos de intersección utilizados durante el recorrido de rayos.
struct Hit<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    object: &'a dyn Intersectable,
}

/// Manejadores de texturas y skyboxes que permanecen válidos durante el render.
#[derive(Copy, Clone)]
pub struct Assets<'a> {
    pub grass_cover: Option<Tex<'a>>,
    pub dirt: Option<Tex<'a>>,
    pub stone: Option<Tex<'a>>,
    pub wood: Option<Tex<'a>>,
    pub leaves: Option<Tex<'a>>,
    pub water: Option<Tex<'a>>,
    pub lava: Option<Tex<'a>>,
    pub obsidian: Option<Tex<'a>>,
    pub glowstone: Option<Tex<'a>>,
    pub diamond: Option<Tex<'a>>,
    pub iron: Option<Tex<'a>>,
    pub chest: Option<Tex<'a>>,
    pub ice: Option<Tex<'a>>,
    pub portal: Option<Tex<'a>>,
    pub skybox_overworld: Option<Skybox<'a>>,
    pub skybox_nether: Option<Skybox<'a>>,
}

/// Geometría ya preparada para renderizar.
pub struct SceneData<'a> {
    pub objects: Vec<DynObject<'a>>,
    pub skybox: Option<Skybox<'a>>,
}
