//! Diorama interactivo inspirado en Minecraft renderizado íntegramente en CPU.

mod camera;
mod math;
mod materials;
mod solid_block;
mod textured_block;
mod grass_block;
mod lighting;
mod raytracer;
mod textured_plane;
mod texture_loader;
mod framebuffer;
mod ray;

use camera::OrbitCamera;
use math::Vec3;
use raylib::prelude::*;
use std::f32::consts::PI;
use texture_loader::TextureStorage;
use raytracer::{Assets, SceneData, WorldKind, render};
use lighting::Skybox;

#[derive(Debug, Clone, Copy, PartialEq)]
enum WorldType {
    Overworld,
    Nether,
}

impl WorldType {
    fn toggle(self) -> Self {
        match self {
            WorldType::Overworld => WorldType::Nether,
            WorldType::Nether => WorldType::Overworld,
        }
    }
}

fn main() {
    // Configurar threads
    let num_threads = num_cpus::get();
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();
    
    println!("=== RAYTRACER CPU - MINECRAFT DIORAMA ===");
    println!("Threads: {}", num_threads);
    
    // Configuración de ventana FULLSCREEN
    let (mut rl, thread) = raylib::init()
        .title("Raytracer CPU - Minecraft Diorama")
        .build();
    
    // Forzar pantalla completa total
    rl.toggle_fullscreen();
    rl.toggle_borderless_windowed();
    rl.set_target_fps(60);
    
    let screen_width = rl.get_screen_width() as u32;
    let screen_height = rl.get_screen_height() as u32;
    
    // OPTIMIZACIÓN CRÍTICA: Reducir resolución para raytracing en CPU
    let render_scale = 0.2; // 20% = mucho más rápido (antes era 50% = MUY LENTO)
    
    let fb_width = ((screen_width as f32 * render_scale) as u32).max(320);
    let fb_height = ((screen_height as f32 * render_scale) as u32).max(180);
    
    println!("Pantalla completa: {}x{}", screen_width, screen_height);
    println!("Framebuffer raytracing: {}x{} ({}%)", 
        fb_width, fb_height, (render_scale * 100.0) as u32);
    
    let img = Image::gen_image_color(fb_width as i32, fb_height as i32, Color::BLACK);
    let mut tex = rl.load_texture_from_image(&thread, &img).expect("texture");
    let mut frame = vec![0u8; (fb_width * fb_height * 4) as usize];
    
    // Cargar texturas PNG para raytracing optimizado
    let textures = TextureStorage::load();
    
    // Crear skybox usando clouds.png para todas las caras
    let clouds_tex = textures.get_clouds();
    let skybox_overworld = Skybox {
        px: clouds_tex,  // +X (derecha)
        nx: clouds_tex,  // -X (izquierda)
        py: clouds_tex,  // +Y (arriba)
        ny: clouds_tex,  // -Y (abajo)
        pz: clouds_tex,  // +Z (frente)
        nz: clouds_tex,  // -Z (atrás)
        tint: Vec3::new(0.8, 0.9, 1.0), // Tinte azul cielo
    };
    
    // Construir assets para raytracing con texturas y skybox
    let assets = Assets {
        grass_cover: Some(textures.get_grass_cover()),
        grass_side: Some(textures.get_grass_side()),
        dirt: Some(textures.get_dirt()),
        stone: Some(textures.get_stone()),
        wood: Some(textures.get_wood()),
        leaves: Some(textures.get_leaves()),
        water: Some(textures.get_water()),
        lava: Some(textures.get_lava()),
        obsidian: Some(textures.get_obsidian()),
        glowstone: Some(textures.get_glowstone()),
        diamond: Some(textures.get_diamond()),
        iron: Some(textures.get_iron()),
        chest: Some(textures.get_chest()),
        ice: Some(textures.get_ice()),
        portal: Some(textures.get_portal()),
        torch: Some(textures.get_torch()),
        skybox_overworld: Some(skybox_overworld),
        skybox_nether: None,  // Nether sin skybox (cielo procedural rojo)
    };
    
    // Construir escenas de raytracing con texturas
    let overworld_rt = raytracer::build_scene(&assets, WorldKind::Overworld);
    let nether_rt = raytracer::build_scene(&assets, WorldKind::Nether);
    
    println!("Overworld (raytracing): {} bloques texturizados", overworld_rt.objects.len());
    println!("Nether (raytracing): {} bloques texturizados", nether_rt.objects.len());
    
    let mut current_world = WorldType::Overworld;
    
    // Cámara orbital
    let mut orbit = OrbitCamera::new(
        0.6,  // yaw
        0.25, // pitch
        25.0, // radius
        Vec3::new(0.0, 2.0, 0.0), // target
    );
    
    // Ciclo solar
    let mut sun_angle: f32 = 0.6; // Ángulo del sol
    let mut animate_sun = false; // ESPACIO para activar
    
    println!("\n=== CONTROLES ===");
    println!("Flechas: Orbitar cámara");
    println!("Q/E: Zoom in/out");
    println!("M: Cambiar mundo (Overworld/Nether)");
    println!("ESPACIO: Ciclo solar día/noche");
    println!("ESC: Salir\n");
    
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        let speed = 1.6;
        
        // Actualizar ciclo solar
        if animate_sun {
            sun_angle += dt * 0.3;
        }
        
        // Control de cámara
        if rl.is_key_down(KeyboardKey::KEY_LEFT) { orbit.yaw -= speed * dt; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { orbit.yaw += speed * dt; }
        if rl.is_key_down(KeyboardKey::KEY_UP) { orbit.pitch -= speed * dt; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) { orbit.pitch += speed * dt; }
        if rl.is_key_down(KeyboardKey::KEY_Q) { orbit.radius = (orbit.radius - 2.0 * dt).max(3.0); }
        if rl.is_key_down(KeyboardKey::KEY_E) { orbit.radius = (orbit.radius + 2.0 * dt).min(50.0); }
        
        orbit.pitch = orbit.pitch.clamp(-PI * 0.48, PI * 0.48);
        
        // Cambiar mundo (M)
        if rl.is_key_pressed(KeyboardKey::KEY_M) {
            current_world = current_world.toggle();
            println!("Mundo: {}", match current_world {
                WorldType::Overworld => "OVERWORLD",
                WorldType::Nether => "NETHER",
            });
        }
        
        // Ciclo solar (ESPACIO)
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            animate_sun = !animate_sun;
            println!("Ciclo solar: {}", if animate_sun { "ACTIVO" } else { "PAUSADO" });
        }
        
        // Seleccionar escena actual (raytracing con texturas)
        let scene_rt = match current_world {
            WorldType::Overworld => &overworld_rt,
            WorldType::Nether => &nether_rt,
        };
        
        let camera = orbit.to_camera(60.0);
        
        // === RENDERIZADO RAYTRACING ===
        
        // Raytracing optimizado en CPU con texturas PNG y ciclo solar
        render(
            &mut frame,
            fb_width as i32,
            fb_height as i32,
            &camera,
            sun_angle, // Pasar el ángulo del sol para iluminación dinámica
            scene_rt,
            1, // max depth REDUCIDO de 2 a 1 para MUCHO mejor FPS (menos reflejos recursivos)
        );
        let _ = tex.update_texture(&frame);

        
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        
        // Escalar textura a pantalla completa con filtrado
        let src = Rectangle::new(0.0, 0.0, fb_width as f32, fb_height as f32);
        let dst = Rectangle::new(0.0, 0.0, d.get_screen_width() as f32, d.get_screen_height() as f32);
        d.draw_texture_pro(&tex, src, dst, Vector2::zero(), 0.0, Color::WHITE);
        
        d.draw_text(&format!("FPS: {} | RAYTRACING | {} | Sol: {}", 
            d.get_fps(),
            match current_world {
                WorldType::Overworld => "OVERWORLD",
                WorldType::Nether => "NETHER",
            },
            if animate_sun { "Animado" } else { "Pausado" }
        ), 10, 10, 24, Color::LIME);
        d.draw_text("M: Cambiar mundo | SPACE: Ciclo solar", 10, 40, 18, Color::YELLOW);
    }
}
