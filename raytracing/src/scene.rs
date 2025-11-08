//! Escena con geometría y luces.

use crate::geometry::{Sphere, Triangle, cube_triangles};
use crate::math::Vec3;
use crate::texture::{Tex, load_texture_rgba};

pub struct Scene {
    pub triangles: Vec<Triangle>,
    pub spheres: Vec<Sphere>,
    pub ambient: Vec3,
    pub sky_color: Vec3, // Color del cielo
}

// Almacenamiento global de texturas para mantenerlas vivas
pub struct TextureStorage {
    pub grass_buf: Option<Vec<u8>>,
    pub grass_wh: (u32, u32),
    pub dirt_buf: Option<Vec<u8>>,
    pub dirt_wh: (u32, u32),
    pub wood_buf: Option<Vec<u8>>,
    pub wood_wh: (u32, u32),
    pub leaves_buf: Option<Vec<u8>>,
    pub leaves_wh: (u32, u32),
    pub sand_buf: Option<Vec<u8>>,
    pub sand_wh: (u32, u32),
    pub water_buf: Option<Vec<u8>>,
    pub water_wh: (u32, u32),
}

impl TextureStorage {
    pub fn load() -> Self {
        println!("Cargando texturas...");
        
        let grass = load_texture_rgba("assets/grass_top.png")
            .or_else(|| load_texture_rgba("assets/cesped.jpg"));
        let dirt = load_texture_rgba("assets/dirt.png")
            .or_else(|| load_texture_rgba("assets/tierra.jpg"));
        let wood = load_texture_rgba("assets/wood.png")
            .or_else(|| load_texture_rgba("assets/madera.jpg"));
        let leaves = load_texture_rgba("assets/leaves.png")
            .or_else(|| load_texture_rgba("assets/hojas.jpg"));
        let sand = load_texture_rgba("assets/sand.png")
            .or_else(|| load_texture_rgba("assets/arena.jpg"));
        let water = load_texture_rgba("assets/water.png")
            .or_else(|| load_texture_rgba("assets/agua.jpg"));
        
        let mut storage = Self {
            grass_buf: None,
            grass_wh: (0, 0),
            dirt_buf: None,
            dirt_wh: (0, 0),
            wood_buf: None,
            wood_wh: (0, 0),
            leaves_buf: None,
            leaves_wh: (0, 0),
            sand_buf: None,
            sand_wh: (0, 0),
            water_buf: None,
            water_wh: (0, 0),
        };
        
        if let Some((buf, w, h)) = grass {
            println!("  ✓ Grass: {}x{}", w, h);
            storage.grass_buf = Some(buf);
            storage.grass_wh = (w, h);
        }
        if let Some((buf, w, h)) = dirt {
            println!("  ✓ Dirt: {}x{}", w, h);
            storage.dirt_buf = Some(buf);
            storage.dirt_wh = (w, h);
        }
        if let Some((buf, w, h)) = wood {
            println!("  ✓ Wood: {}x{}", w, h);
            storage.wood_buf = Some(buf);
            storage.wood_wh = (w, h);
        }
        if let Some((buf, w, h)) = leaves {
            println!("  ✓ Leaves: {}x{}", w, h);
            storage.leaves_buf = Some(buf);
            storage.leaves_wh = (w, h);
        }
        if let Some((buf, w, h)) = sand {
            println!("  ✓ Sand: {}x{}", w, h);
            storage.sand_buf = Some(buf);
            storage.sand_wh = (w, h);
        }
        if let Some((buf, w, h)) = water {
            println!("  ✓ Water: {}x{}", w, h);
            storage.water_buf = Some(buf);
            storage.water_wh = (w, h);
        }
        
        storage
    }
    
    pub fn get_grass(&self) -> Option<Tex> {
        self.grass_buf.as_ref().map(|buf| Tex {
            pix: &buf[..],
            w: self.grass_wh.0,
            h: self.grass_wh.1,
        })
    }
    
    pub fn get_wood(&self) -> Option<Tex> {
        self.wood_buf.as_ref().map(|buf| Tex {
            pix: &buf[..],
            w: self.wood_wh.0,
            h: self.wood_wh.1,
        })
    }
    
    pub fn get_leaves(&self) -> Option<Tex> {
        self.leaves_buf.as_ref().map(|buf| Tex {
            pix: &buf[..],
            w: self.leaves_wh.0,
            h: self.leaves_wh.1,
        })
    }
    
    pub fn get_sand(&self) -> Option<Tex> {
        self.sand_buf.as_ref().map(|buf| Tex {
            pix: &buf[..],
            w: self.sand_wh.0,
            h: self.sand_wh.1,
        })
    }
    
    pub fn get_water(&self) -> Option<Tex> {
        self.water_buf.as_ref().map(|buf| Tex {
            pix: &buf[..],
            w: self.water_wh.0,
            h: self.water_wh.1,
        })
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            triangles: Vec::new(),
            spheres: Vec::new(),
            ambient: Vec3::new(0.2, 0.2, 0.25),
            sky_color: Vec3::new(0.5, 0.7, 1.0), // Azul por defecto
        }
    }

    pub fn add_cube(&mut self, center: Vec3, size: f32, albedo: Vec3) {
        self.triangles.extend(cube_triangles(center, size, albedo));
    }

    pub fn add_sphere(&mut self, center: Vec3, radius: f32, albedo: Vec3, emissive: Vec3) {
        self.spheres.push(Sphere::new(center, radius, albedo, emissive));
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum WorldType {
    Overworld,
    Nether,
}

impl WorldType {
    pub fn toggle(&self) -> Self {
        match self {
            WorldType::Overworld => WorldType::Nether,
            WorldType::Nether => WorldType::Overworld,
        }
    }
}

/// Construye el mundo Overworld (diorama Minecraft).
pub fn build_overworld() -> Scene {
    let mut scene = Scene::new();
    scene.sky_color = Vec3::new(0.5, 0.7, 1.0); // Cielo azul
    
    // Colores
    let grass = Vec3::new(0.4, 0.7, 0.3);
    let wood = Vec3::new(0.55, 0.35, 0.17);
    let leaves = Vec3::new(0.2, 0.6, 0.2);
    let sand = Vec3::new(0.93, 0.79, 0.69);
    let water = Vec3::new(0.2, 0.4, 0.8);
    
    // Suelo de grama (6x6)
    for x in -3..=2 {
        for z in -2..=3 {
            let pos = Vec3::new(x as f32 * 2.0, -1.0, z as f32 * 2.0);
            scene.add_cube(pos, 2.0, grass);
        }
    }
    
    // Árbol - tronco (3 cubos verticales)
    scene.add_cube(Vec3::new(-2.0, 1.0, -2.0), 2.0, wood);
    scene.add_cube(Vec3::new(-2.0, 3.0, -2.0), 2.0, wood);
    scene.add_cube(Vec3::new(-2.0, 5.0, -2.0), 2.0, wood);
    
    // Árbol - copa (forma de cruz en 3 niveles)
    let leaf_positions = vec![
        // Nivel 1 (y=7.0)
        Vec3::new(-2.0, 7.0, -2.0), // centro
        Vec3::new(-4.0, 7.0, -2.0), // izquierda
        Vec3::new(0.0, 7.0, -2.0),  // derecha
        Vec3::new(-2.0, 7.0, -4.0), // atrás
        Vec3::new(-2.0, 7.0, 0.0),  // adelante
        // Nivel 2 (y=9.0)
        Vec3::new(-2.0, 9.0, -2.0),
        Vec3::new(-4.0, 9.0, -2.0),
        Vec3::new(0.0, 9.0, -2.0),
        // Nivel 3 (y=11.0)
        Vec3::new(-2.0, 11.0, -2.0),
    ];
    
    for pos in leaf_positions {
        scene.add_cube(pos, 2.0, leaves);
    }
    
    // Playa de arena (4x4 alrededor del lago)
    for x in 3..=6 {
        for z in 0..=3 {
            let is_lake = x >= 4 && x <= 5 && z >= 1 && z <= 2;
            if !is_lake {
                let pos = Vec3::new(x as f32 * 2.0, -1.0, z as f32 * 2.0);
                scene.add_cube(pos, 2.0, sand);
            }
        }
    }
    
    // Lago (2x2)
    for x in 4..=5 {
        for z in 1..=2 {
            let pos = Vec3::new(x as f32 * 2.0, -0.5, z as f32 * 2.0);
            scene.add_cube(pos, 2.0, water);
        }
    }
    
    // Antorchas (esferas emisivas amarillas)
    let torch_color = Vec3::new(0.9, 0.7, 0.2);
    let torch_glow = Vec3::new(1.0, 0.8, 0.3);
    scene.add_sphere(Vec3::new(-5.0, 2.0, -2.0), 0.3, torch_color, torch_glow);
    scene.add_sphere(Vec3::new(8.0, 2.0, 3.0), 0.3, torch_color, torch_glow);
    
    // Sol (esfera emisiva grande)
    let sun_color = Vec3::new(1.0, 1.0, 0.4);
    let sun_glow = Vec3::new(1.5, 1.5, 0.8);
    scene.add_sphere(Vec3::new(15.0, 18.0, -20.0), 3.0, sun_color, sun_glow);
    
    scene
}

/// Construye el mundo Nether con obsidiana y portal.
pub fn build_nether() -> Scene {
    let mut scene = Scene::new();
    scene.sky_color = Vec3::new(0.3, 0.1, 0.1); // Cielo rojo oscuro
    scene.ambient = Vec3::new(0.15, 0.05, 0.05); // Ambient rojizo
    
    // Colores
    let obsidian = Vec3::new(0.1, 0.05, 0.15); // Negro morado
    let netherrack = Vec3::new(0.4, 0.2, 0.2); // Rojo oscuro
    let lava_glow = Vec3::new(1.0, 0.3, 0.0); // Naranja brillante
    
    // Suelo de obsidiana (6x6)
    for x in -3..=2 {
        for z in -2..=3 {
            let pos = Vec3::new(x as f32 * 2.0, -1.0, z as f32 * 2.0);
            scene.add_cube(pos, 2.0, obsidian);
        }
    }
    
    // Portal en el centro (estructura vertical)
    // Base del portal (4x3 horizontal)
    for x in -1..=0 {
        for z in -1..=1 {
            let pos = Vec3::new(x as f32 * 2.0, -1.0, z as f32 * 2.0);
            scene.add_cube(pos, 2.0, obsidian);
        }
    }
    
    // Pilares laterales del portal (altura 6)
    for y in 0..6 {
        // Pilar izquierdo
        scene.add_cube(Vec3::new(-2.0, 1.0 + y as f32 * 2.0, -2.0), 2.0, obsidian);
        scene.add_cube(Vec3::new(-2.0, 1.0 + y as f32 * 2.0, 0.0), 2.0, obsidian);
        scene.add_cube(Vec3::new(-2.0, 1.0 + y as f32 * 2.0, 2.0), 2.0, obsidian);
        
        // Pilar derecho
        scene.add_cube(Vec3::new(2.0, 1.0 + y as f32 * 2.0, -2.0), 2.0, obsidian);
        scene.add_cube(Vec3::new(2.0, 1.0 + y as f32 * 2.0, 0.0), 2.0, obsidian);
        scene.add_cube(Vec3::new(2.0, 1.0 + y as f32 * 2.0, 2.0), 2.0, obsidian);
    }
    
    // Techo del portal
    for x in -1..=1 {
        for z in -1..=1 {
            scene.add_cube(Vec3::new(x as f32 * 2.0, 13.0, z as f32 * 2.0), 2.0, obsidian);
        }
    }
    
    // Particulas moradas flotantes (esferas pequeñas)
    let purple = Vec3::new(0.5, 0.2, 0.8);
    let purple_glow = Vec3::new(0.6, 0.3, 1.0);
    
    // Partículas dispersas
    scene.add_sphere(Vec3::new(0.0, 8.0, 0.0), 0.2, purple, purple_glow);
    scene.add_sphere(Vec3::new(-4.0, 5.0, -3.0), 0.2, purple, purple_glow);
    scene.add_sphere(Vec3::new(5.0, 6.0, 2.0), 0.2, purple, purple_glow);
    scene.add_sphere(Vec3::new(-3.0, 4.0, 4.0), 0.2, purple, purple_glow);
    scene.add_sphere(Vec3::new(4.0, 7.0, -4.0), 0.2, purple, purple_glow);
    scene.add_sphere(Vec3::new(1.0, 9.0, 3.0), 0.2, purple, purple_glow);
    scene.add_sphere(Vec3::new(-5.0, 3.0, 1.0), 0.2, purple, purple_glow);
    
    // Torres de netherrack decorativas
    for y in 0..4 {
        scene.add_cube(Vec3::new(6.0, 1.0 + y as f32 * 2.0, 4.0), 2.0, netherrack);
        scene.add_cube(Vec3::new(-6.0, 1.0 + y as f32 * 2.0, -4.0), 2.0, netherrack);
    }
    
    // Lava/fuego (esferas naranjas brillantes)
    scene.add_sphere(Vec3::new(-5.0, 1.0, 3.0), 0.4, lava_glow, lava_glow.mul(1.5));
    scene.add_sphere(Vec3::new(7.0, 1.0, -3.0), 0.4, lava_glow, lava_glow.mul(1.5));
    
    // "Sol" del Nether (rojo oscuro)
    let nether_sun = Vec3::new(0.8, 0.2, 0.2);
    let nether_glow = Vec3::new(1.0, 0.3, 0.3);
    scene.add_sphere(Vec3::new(10.0, 15.0, -15.0), 2.5, nether_sun, nether_glow);
    
    scene
}
