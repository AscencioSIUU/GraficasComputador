//! Construye la escena de bloques y ejecuta el trazador de rayos en CPU.

use std::collections::HashSet;
use std::thread;

use crate::lighting::{Skybox, Tex, reflect, refract, sample_skybox, sky, specular_phong, to_rgba};
use crate::camera::Camera;
use crate::solid_block::SolidBlock;
use crate::textured_block::TexturedBlock;
use crate::grass_block::GrassBlock;
use crate::math::Vec3;
use crate::ray::Ray;
use crate::materials::Intersectable;

type DynObject<'a> = Box<dyn Intersectable + Send + Sync + 'a>;

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

struct Hit<'a> {
    t: f32,
    point: Vec3,
    normal: Vec3,
    object: &'a dyn Intersectable,
}

#[derive(Copy, Clone)]
pub struct Assets<'a> {
    pub grass_cover: Option<Tex<'a>>,
    pub grass_side: Option<Tex<'a>>,
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
    pub torch: Option<Tex<'a>>,
    pub skybox_overworld: Option<Skybox<'a>>,
    pub skybox_nether: Option<Skybox<'a>>,
}

pub struct SceneData<'a> {
    pub objects: Vec<DynObject<'a>>,
    pub skybox: Option<Skybox<'a>>,
    pub is_nether: bool,  // Indica si es el mundo Nether
}

fn push_block<'a>(objects: &mut Vec<DynObject<'a>>, min: Vec3, max: Vec3, mat: BlockMaterial<'a>) {
    let inner = SolidBlock {
        min,
        max,
        albedo_color: mat.albedo,
        specular_strength: mat.specular,
        shininess: mat.shininess,
        reflectivity: mat.reflectivity,
        transparency: mat.transparency,
        ior: mat.ior,
        emissive: mat.emissive,
    };

    if let Some(t) = mat.tex {
        objects.push(Box::new(TexturedBlock::from_raw(
            inner,
            t.pix,
            t.w,
            t.h,
            mat.specular,
            mat.shininess,
            mat.reflectivity,
            mat.transparency,
            mat.ior,
            mat.emissive,
        )));
    } else {
        objects.push(Box::new(inner));
    }
}

#[derive(Copy, Clone)]
struct BlockMaterial<'a> {
    tex: Option<Tex<'a>>,
    albedo: Vec3,
    specular: f32,
    shininess: f32,
    reflectivity: f32,
    transparency: f32,
    ior: f32,
    emissive: Vec3,
}

impl<'a> BlockMaterial<'a> {
    fn place(&self, objects: &mut Vec<DynObject<'a>>, x: i32, y: i32, z: i32) {
        let min = Vec3::new(x as f32 - 0.5, y as f32 - 0.5, z as f32 - 0.5);
        let max = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);
        push_block(objects, min, max, *self);
    }

    fn place_cover(
        &self,
        objects: &mut Vec<DynObject<'a>>,
        x: i32,
        y: i32,
        z: i32,
        thickness: f32,
    ) {
        let top = y as f32 + 0.5;
        let min = Vec3::new(x as f32 - 0.5, top - thickness, z as f32 - 0.5);
        let max = Vec3::new(x as f32 + 0.5, top, z as f32 + 0.5);
        push_block(objects, min, max, *self);
    }
}

fn place_with_tag<'a>(
    objects: &mut Vec<DynObject<'a>>,
    used: &mut HashSet<(i32, i32, i32, u8)>,
    mat: BlockMaterial<'a>,
    x: i32,
    y: i32,
    z: i32,
    tag: u8,
) {
    if used.insert((x, y, z, tag)) {
        mat.place(objects, x, y, z);
    }
}

fn place_block<'a>(
    objects: &mut Vec<DynObject<'a>>,
    used: &mut HashSet<(i32, i32, i32, u8)>,
    mat: BlockMaterial<'a>,
    x: i32,
    y: i32,
    z: i32,
) {
    place_with_tag(objects, used, mat, x, y, z, 0);
}

/// Coloca un bloque de césped con grass_top arriba y grass_side en los lados
fn place_grass_block<'a>(
    objects: &mut Vec<DynObject<'a>>,
    used: &mut HashSet<(i32, i32, i32, u8)>,
    top_mat: BlockMaterial<'a>,
    side_mat: BlockMaterial<'a>,
    x: i32,
    y: i32,
    z: i32,
) {
    let key = (x, y, z, 0);
    if used.contains(&key) {
        return;
    }
    used.insert(key);

    // ARREGLADO: Usar el mismo tamaño que los demás bloques (1.0×1.0×1.0)
    let min = Vec3::new(x as f32 - 0.5, y as f32 - 0.5, z as f32 - 0.5);
    let max = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5);

    let solid = SolidBlock {
        min,
        max,
        albedo_color: Vec3::new(0.9, 0.95, 0.85),
        specular_strength: 0.08,
        shininess: 20.0,
        reflectivity: 0.01,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };

    // Obtener las texturas
    let top_tex = top_mat.tex.unwrap();
    let side_tex = side_mat.tex.unwrap();

    let grass = GrassBlock::new(
        solid,
        top_tex.pix,
        top_tex.w,
        top_tex.h,
        side_tex.pix,
        side_tex.w,
        side_tex.h,
        0.08,  // specular_strength
        20.0,  // shininess
        0.01,  // reflectivity
        0.0,   // transparency
        1.0,   // ior
        Vec3::new(0.0, 0.0, 0.0), // emissive
    );

    objects.push(Box::new(grass));
}

fn trace<'a>(
    ray: &Ray,
    objects: &'a [DynObject<'a>],
    light_pos: Vec3,
    sun_brightness: f32, // Intensidad del sol (0.1 a 1.0)
    depth: i32,
    skybox: Option<&Skybox<'a>>,
    is_nether: bool, // Flag para indicar si es el mundo Nether
) -> Vec3 {
    let mut closest: Option<Hit> = None;
    for o in objects.iter() {
        if let Some(t) = o.intersect(ray) {
            if t > 0.0 && (closest.is_none() || t < closest.as_ref().unwrap().t) {
                let p = ray.orig.add(ray.dir.mul(t));
                let n = o.normal_at(p);
                closest = Some(Hit {
                    t,
                    point: p,
                    normal: n,
                    object: o.as_ref(),
                });
            }
        }
    }

    if closest.is_none() {
        // Color del cielo modulado por el brillo del sol
        let sky_color = if let Some(sb) = skybox {
            sample_skybox(ray.dir, sb)
        } else {
            sky(ray.dir, is_nether)
        };
        
        // En el Nether, no modular tanto por el sol (siempre rojo)
        if is_nether {
            return sky_color.mul(0.9); // Mantener el rojo constante
        } else {
            return sky_color.mul(sun_brightness * 0.8 + 0.2); // Entre 20% y 100%
        }
    }

    let hit = closest.unwrap();
    let mat = hit.object.material_at(hit.point);

    // Constante para evitar auto-intersección
    let bias = 1e-3;
    
    // Iluminación - calcular normal y dirección de luz primero
    let n = hit.normal.norm();
    let ldir = light_pos.sub(hit.point).norm();
    let ndotl = n.dot(ldir).max(0.0);
    
    // OPTIMIZACIÓN: Solo calcular sombras si el objeto está orientado hacia la luz
    // y hay suficiente brillo solar (evita cálculos costosos en la noche o caras traseras)
    let mut in_shadow = false;
    if ndotl > 0.01 && sun_brightness > 0.15 {
        let shadow_origin = hit.point.add(hit.normal.mul(bias));
        let light_distance = light_pos.sub(hit.point).len();
        let sray = Ray {
            orig: shadow_origin,
            dir: ldir,
        };
        
        for o in objects.iter() {
            if let Some(t) = o.intersect(&sray) {
                if t < light_distance {
                    in_shadow = true;
                    break;
                }
            }
        }
    }

    // Iluminación modulada por el ciclo solar
    let ambient = 0.05 * sun_brightness; // Ambiente varía con el sol
    let v = (-ray.dir).norm();
    let mut local = mat
        .albedo
        .mul(ambient + if in_shadow { 0.0 } else { ndotl * sun_brightness });

    if !in_shadow && ndotl > 0.0 && mat.specular_strength > 0.0 {
        let r = reflect(ldir, n);
        let spec = specular_phong(r, v, mat.specular_strength, mat.shininess) * sun_brightness;
        local = local.add(Vec3::new(spec, spec, spec));
    }

    // Añadir emisión propia del material (para portal)
    local = local.add(mat.emissive);

    if depth <= 0 {
        return local;
    }

    let mut accum = Vec3::new(0.0, 0.0, 0.0);
    let mut weight = 1.0;

    if mat.transparency > 0.0 {
        let mut n_out = n;
        let mut eta = 1.0 / mat.ior;
        if ray.dir.dot(n) > 0.0 {
            n_out = -n;
            eta = mat.ior;
        }
        if let Some(tdir) = refract(ray.dir, n_out, eta) {
            let ro = hit.point.add(tdir.mul(bias));
            let rr = Ray {
                orig: ro,
                dir: tdir,
            };
            let refr_col = trace(&rr, objects, light_pos, sun_brightness, depth - 1, skybox, is_nether);
            accum = accum.add(refr_col.mul(mat.transparency));
            weight -= mat.transparency;
        }
    }

    if mat.reflectivity > 0.0 && weight > 0.0 {
        let rdir = reflect(ray.dir, n).norm();
        let ro = hit.point.add(n.mul(bias));
        let rr = Ray {
            orig: ro,
            dir: rdir,
        };
        let refl_col = trace(&rr, objects, light_pos, sun_brightness, depth - 1, skybox, is_nether);
        accum = accum.add(refl_col.mul(mat.reflectivity));
        weight -= mat.reflectivity;
    }

    local.mul(weight.max(0.0)).add(accum)
}

pub fn build_scene<'a>(assets: &Assets<'a>, world: WorldKind) -> SceneData<'a> {
    let mut objects: Vec<DynObject<'a>> = Vec::new();
    let mut used: HashSet<(i32, i32, i32, u8)> = HashSet::new();
    
    // Definir materiales con RAYTRACING mejorado
    let grass_top_mat = BlockMaterial {
        tex: assets.grass_cover,
        albedo: Vec3::new(0.95, 1.0, 0.95),
        specular: 0.15,        // Aumentado de 0.08 para reflejos especulares visibles
        shininess: 35.0,       // Aumentado de 20.0 para reflejos más definidos
        reflectivity: 0.08,    // Aumentado de 0.01 para reflejos ambientales
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    
    let grass_side_mat = BlockMaterial {
        tex: assets.grass_side,
        albedo: Vec3::new(0.90, 0.95, 0.85),
        specular: 0.12,        // Aumentado de 0.06
        shininess: 28.0,       // Aumentado de 18.0
        reflectivity: 0.06,    // Aumentado de 0.01
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    
    let dirt_mat = BlockMaterial {
        tex: assets.dirt,
        albedo: Vec3::new(0.85, 0.76, 0.6),
        specular: 0.02,
        shininess: 10.0,
        reflectivity: 0.0,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    
    let wood_mat = BlockMaterial {
        tex: assets.wood,
        albedo: Vec3::new(1.0, 0.98, 0.92),
        specular: 0.18,        // Aumentado de 0.04 - madera con barniz reflectante
        shininess: 40.0,       // Aumentado de 18.0 para reflejos más brillantes
        reflectivity: 0.12,    // Aumentado de 0.01 para reflejos visibles
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    
    let leaves_mat = BlockMaterial {
        tex: assets.leaves,
        albedo: Vec3::new(0.6, 1.0, 0.6),
        specular: 0.20,        // Aumentado de 0.05 - hojas brillantes/húmedas
        shininess: 25.0,       // Aumentado de 15.0
        reflectivity: 0.10,    // Aumentado de 0.02 para reflejos de luz
        transparency: 0.25,    // Aumentado de 0.15 para más translucidez
        ior: 1.08,             // Aumentado de 1.05 para refracción más visible
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    
    let obsidian_mat = BlockMaterial {
        tex: assets.obsidian,
        albedo: Vec3::new(0.6, 0.65, 0.8),
        specular: 0.18,
        shininess: 70.0,
        reflectivity: 0.08,
        transparency: 0.0,
        ior: 1.46,
        emissive: Vec3::new(0.0, 0.0, 0.0),
    };
    
    let portal_mat = BlockMaterial {
        tex: assets.portal,
        albedo: Vec3::new(1.0, 0.4, 1.2),
        specular: 0.6,
        shininess: 60.0,
        reflectivity: 0.12,
        transparency: 0.55,
        ior: 1.6,
        emissive: Vec3::new(1.5, 0.3, 1.8),
    };
    
    // Construir mundo simple
    match world {
        WorldKind::Overworld => {
            // Piso de césped 11x8 según diseño:
            // NUEVO DISEÑO OVERWORLD 11x8 - MÁS ESPACIO Y AMPLITUD
            // Fila 1 (z=-3):  🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩  ← sin árboles
            // Fila 2 (z=-2):  🟩🌲🟩🟩🌲🟩🟩🟩🟩🌲🟩  ← árboles en x=-4, -1, 4
            // Fila 3 (z=-1):  🟩🟩🟩🟩🟩🟩🟩🌲🟩🟩🟩  ← árbol en x=3
            // Fila 4 (z=0):   🟩🟩🟩🟩⬛️🟪⬛️🟩🟩🟩🟩  ← portal en x=0,1,2
            // Fila 5 (z=1):   🟩🟩🟩🌲🟩🟩🟩🟩🟩🟩🟩  ← árbol en x=-3
            // Fila 6 (z=2):   🟩🌲🟩🟩🟩🟩🟩🟩🟩🌲🟩  ← árboles en x=-4, 4
            // Fila 7 (z=3):   🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩  ← sin árboles
            // Fila 8 (z=4):   🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩  ← sin árboles
            
            // Piso de pasto completo 11x8
            for x in -5..=5 {
                for z in -3..=4 {
                    place_grass_block(&mut objects, &mut used, grass_top_mat, grass_side_mat, x, 0, z);
                }
            }
            
            // Helper para crear un árbol completo en una posición
            // OPTIMIZACIÓN: Árboles más pequeños (altura 5 en vez de 6) para mejor FPS
            let place_tree = |objects: &mut Vec<Box<dyn Intersectable + Send + Sync>>, used: &mut HashSet<(i32, i32, i32, u8)>, x: i32, z: i32| {
                // Tronco de 3 bloques (reducido de 4)
                for y in 1..=3 {
                    place_with_tag(objects, used, wood_mat, x, y, z, 1);
                }
                // Hojas capa inferior (y=3)
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        if dx != 0 || dz != 0 {
                            place_with_tag(objects, used, leaves_mat, x + dx, 3, z + dz, 1);
                        }
                    }
                }
                // Hojas capa media (y=4)
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        place_with_tag(objects, used, leaves_mat, x + dx, 4, z + dz, 1);
                    }
                }
                // Hojas capa superior (y=5) - solo 5 bloques en cruz
                place_with_tag(objects, used, leaves_mat, x, 5, z, 1);
                place_with_tag(objects, used, leaves_mat, x - 1, 5, z, 1);
                place_with_tag(objects, used, leaves_mat, x + 1, 5, z, 1);
                place_with_tag(objects, used, leaves_mat, x, 5, z - 1, 1);
                place_with_tag(objects, used, leaves_mat, x, 5, z + 1, 1);
            };
            
            // ÁRBOLES distribuidos al azar - 7 árboles optimizados para FPS
            // Distribución en 9x7: mismas posiciones pero con más espacio
            
            // Fila 2 (z=-2): 3 árboles 🌲🟩🟩🌲🟩🟩🟩🟩🌲
            place_tree(&mut objects, &mut used, -4, -2);
            place_tree(&mut objects, &mut used, -1, -2);
            place_tree(&mut objects, &mut used, 4, -2);
            
            // Fila 3 (z=-1): 1 árbol 🟩🟩🟩🟩🟩🟩🌲🟩🟩
            place_tree(&mut objects, &mut used, 3, -1);
            
            // Fila 5 (z=1): 1 árbol 🟩🟩🌲🟩🟩🟩🟩🟩🟩
            place_tree(&mut objects, &mut used, -3, 1);
            
            // Fila 6 (z=2): 2 árboles 🌲🟩🟩🟩🟩🟩🟩🟩🌲
            place_tree(&mut objects, &mut used, -4, 2);
            place_tree(&mut objects, &mut used, 4, 2);
            
            // PORTAL (⬛️🟪⬛️) en z=0, x=0,1,2 (centro fila 3)
            // Pilar izquierdo
            for y in 0..=4 {
                place_with_tag(&mut objects, &mut used, obsidian_mat, 0, y, 0, 1);
            }
            // Pilar derecho
            for y in 0..=4 {
                place_with_tag(&mut objects, &mut used, obsidian_mat, 2, y, 0, 1);
            }
            // Base y techo
            place_with_tag(&mut objects, &mut used, obsidian_mat, 1, 0, 0, 1);
            place_with_tag(&mut objects, &mut used, obsidian_mat, 1, 4, 0, 1);
            // Interior portal
            place_with_tag(&mut objects, &mut used, portal_mat, 1, 1, 0, 1);
            place_with_tag(&mut objects, &mut used, portal_mat, 1, 2, 0, 1);
            place_with_tag(&mut objects, &mut used, portal_mat, 1, 3, 0, 1);
        }
        WorldKind::Nether => {
            // Piso de obsidiana 9x5
            // ◼️⬛️⬛️⬛️⬛️⬛️⬛️⬛️◼️ (Fila 1)
            // ⬛️⬛️⬛️⬛️⬛️⬛️◼️⬛️⬛️ (Fila 2)
            // ⬛️⬛️🟪⬛️⬛️⬛️⬛️⬛️⬛️ (Fila 3)
            // ⬛️⬛️⬛️⬛️⬛️◼️⬛️⬛️⬛️ (Fila 4)
            // ◼️⬛️⬛️⬛️⬛️⬛️⬛️⬛️◼️ (Fila 5)
            
            // Piso completo de obsidiana (9x5)
            for x in -4..=4 {  // 9 bloques de ancho
                for z in -2..=2 { // 5 bloques de profundidad
                    place_block(&mut objects, &mut used, obsidian_mat, x, 0, z);
                }
            }
            
            // OPTIMIZACIÓN: Pilares reducidos de altura 5 a 3 para mejor FPS
            let pillar_height = 3;
            
            // Fila 1: Esquina izquierda (-4, -2) y derecha (4, -2)
            for y in 1..=pillar_height {
                place_with_tag(&mut objects, &mut used, obsidian_mat, -4, y, -2, 2);
                place_with_tag(&mut objects, &mut used, obsidian_mat, 4, y, -2, 2);
            }
            
            // Fila 2: Posición (2, -1) - pilar
            for y in 1..=pillar_height {
                place_with_tag(&mut objects, &mut used, obsidian_mat, 2, y, -1, 2);
            }
            
            // Fila 4: Posición (1, 1) - pilar
            for y in 1..=pillar_height {
                place_with_tag(&mut objects, &mut used, obsidian_mat, 1, y, 1, 2);
            }
            
            // Fila 5: Esquina izquierda (-4, 2) y derecha (4, 2)
            for y in 1..=pillar_height {
                place_with_tag(&mut objects, &mut used, obsidian_mat, -4, y, 2, 2);
                place_with_tag(&mut objects, &mut used, obsidian_mat, 4, y, 2, 2);
            }
            
            // Portal en la tercera fila (🟪) - posición (-2, 0)
            // Marco del portal (obsidiana) - altura reducida a 3 para optimización
            for y in 0..=3 {
                place_with_tag(&mut objects, &mut used, obsidian_mat, -3, y, 0, 2); // Pilar izquierdo
                place_with_tag(&mut objects, &mut used, obsidian_mat, -1, y, 0, 2); // Pilar derecho
            }
            
            // Base y techo del portal
            place_with_tag(&mut objects, &mut used, obsidian_mat, -2, 0, 0, 2);
            place_with_tag(&mut objects, &mut used, obsidian_mat, -2, 3, 0, 2);
            
            // Interior del portal (🟪) - bloques morados emisivos
            place_with_tag(&mut objects, &mut used, portal_mat, -2, 1, 0, 2);
            place_with_tag(&mut objects, &mut used, portal_mat, -2, 2, 0, 2);
        }
    }
    
    let skybox = match world {
        WorldKind::Overworld => assets.skybox_overworld,
        WorldKind::Nether => assets.skybox_nether,
    };
    
    let is_nether = matches!(world, WorldKind::Nether);

    SceneData { objects, skybox, is_nether }
}

pub fn render<'a>(
    frame: &mut [u8],
    w: i32,
    h: i32,
    cam: &Camera,
    sun_angle: f32, // Ángulo del sol para ciclo día/noche
    scene: &SceneData<'a>,
    max_depth: i32,
) {
    let aspect = w as f32 / h as f32;
    let width = w as usize;
    let height = h as usize;
    
    // Calcular posición del sol basada en el ángulo
    let sun_distance = 20.0;
    let light_pos = Vec3::new(
        sun_distance * sun_angle.cos(),
        10.0 + 8.0 * sun_angle.sin().max(0.0),
        sun_distance * sun_angle.sin(),
    );
    
    // Calcular brillo del sol (0.1 a 1.0)
    let sun_brightness = (sun_angle.sin() * 0.5 + 0.5).max(0.1);

    let threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(height.max(1));
    let rows_per_chunk = (height + threads - 1) / threads;
    let pixels_per_row = width * 4;
    let objects_ref: &[DynObject<'a>] = &scene.objects;
    let skybox = scene.skybox.as_ref();
    let is_nether = scene.is_nether;

    thread::scope(|scope| {
        let mut start_row = 0usize;
        let mut remaining: &mut [u8] = frame;
        for _ in 0..threads {
            if start_row >= height {
                break;
            }
            let rows_left = height - start_row;
            let rows_here = rows_per_chunk.min(rows_left);
            let bytes_here = rows_here * pixels_per_row;
            let (chunk, rest) = remaining.split_at_mut(bytes_here);
            let chunk_start = start_row;
            remaining = rest;
            let cam_ref = cam;
            scope.spawn(move || {
                for (row_offset, row) in chunk.chunks_mut(pixels_per_row).enumerate() {
                    let y = (chunk_start + row_offset) as i32;
                    let v = (y as f32 + 0.5) / h as f32;
                    for x in 0..width {
                        let u = (x as f32 + 0.5) / w as f32;
                        let ray = cam_ref.make_ray(u, v, aspect);
                        let color = trace(&ray, objects_ref, light_pos, sun_brightness, max_depth, skybox, is_nether);
                        let idx = x * 4;
                        row[idx..idx + 4].copy_from_slice(&to_rgba(color));
                    }
                }
            });
            start_row += rows_here;
        }
    });
}
