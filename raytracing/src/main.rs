mod framebuffer;
mod raytracer;
mod materials;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use raytracer::*;
use materials::{MaterialType};

fn main() {
    let screen_width = 800;
    let screen_height = 600;
    
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("Raytracing Diorama - Minecraft Style")
        .build();
    
    rl.set_target_fps(60);
    
    let mut fb = Framebuffer::new(screen_width, screen_height, Color::BLACK);
    let mut camera = Camera::new();
    let mut time: f32 = 0.0;
    let mut time_speed: f32 = 1.0;
    let mut paused = false;
    
    println!("=== Cargando escena ===");
    let mut scene = create_diorama(0.0);
    println!("=== Escena cargada con {} triángulos y {} esferas ===", 
             scene.triangles.len(), scene.spheres.len());
    
    println!("\n=== CONTROLES ===");
    println!("W/S: Acercar/Alejar cámara");
    println!("A/D: Rotar cámara horizontalmente");
    println!("Q/E: Rotar cámara verticalmente");
    println!("Espacio: Pausar/Reanudar ciclo día/noche");
    println!("T: Acelerar tiempo");
    println!("R: Reset cámara");
    println!("ESC: Salir\n");
    
    let mut frame_count = 0;
    let mut fps_timer = 0.0;
    
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        fps_timer += dt;
        frame_count += 1;
        
        // Mostrar FPS cada segundo
        if fps_timer >= 1.0 {
            println!("FPS: {} | Time: {:.1} | Cam distance: {:.1}", 
                     frame_count, time, camera.distance);
            fps_timer = 0.0;
            frame_count = 0;
        }
        
        // Update time
        if !paused {
            time += dt * time_speed;
        }
        
        // Camera controls
        if rl.is_key_down(KeyboardKey::KEY_W) {
            camera.distance = (camera.distance - 10.0 * dt).max(5.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            camera.distance = (camera.distance + 10.0 * dt).min(50.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            camera.yaw -= 2.0 * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            camera.yaw += 2.0 * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            camera.pitch = (camera.pitch + 1.5 * dt).min(1.5);
        }
        if rl.is_key_down(KeyboardKey::KEY_E) {
            camera.pitch = (camera.pitch - 1.5 * dt).max(-1.5);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            paused = !paused;
            println!("Ciclo día/noche: {}", if paused { "PAUSADO" } else { "ACTIVO" });
        }
        if rl.is_key_pressed(KeyboardKey::KEY_T) {
            time_speed *= 2.0;
            if time_speed > 8.0 { time_speed = 0.25; }
            println!("Velocidad de tiempo: {}x", time_speed);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            camera = Camera::new();
            time = 0.0;
            paused = false;
            println!("Cámara reseteada");
        }
        
        // Update scene with animated textures
        scene = create_diorama(time);
        scene.update_time(time);
        
        // Render
        fb.clear();
        render_parallel(&mut fb, &scene, &camera);
        
        // Draw to screen
        let texture = rl.load_texture_from_image(&thread, &fb.color_buffer)
            .expect("Failed to create texture");
        
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
        
        // UI info
        d.draw_text(&format!("FPS: {}", d.get_fps()), 10, 10, 20, Color::YELLOW);
        d.draw_text(&format!("Time: {:.1}s", time), 10, 35, 20, Color::YELLOW);
        d.draw_text(
            if paused { "PAUSED" } else { "RUNNING" },
            10, 60, 20,
            if paused { Color::RED } else { Color::GREEN }
        );
    }
}

fn create_diorama(time: f32) -> Scene {
    let mut scene = Scene::new();
    
    // === Texturas animadas ===
    let water_tex = ImageTexture::animated_water(time);
    let portal_tex = ImageTexture::animated_portal(time);
    
    // === Materiales ===
    let grass = Material::with_texture(
        MaterialType::Grass,
        ImageTexture::from_file_or_fallback("assets/grass_top.png", Color::new(100, 180, 70, 255))
    );
    let grass_side = Material::with_texture(
        MaterialType::Grass,
        ImageTexture::from_file_or_fallback("assets/grass_side.png", Color::new(100, 140, 70, 255))
    );
    let dirt = Material::with_texture(
        MaterialType::Dirt,
        ImageTexture::from_file_or_fallback("assets/dirt.png", Color::new(134, 96, 67, 255))
    );
    let stone = Material::with_texture(
        MaterialType::Stone,
        ImageTexture::from_file_or_fallback("assets/stone.png", Color::new(128, 128, 128, 255))
    );
    let water = Material::with_texture(MaterialType::Water, water_tex);
    let glass = Material::with_texture(
        MaterialType::Glass,
        ImageTexture::from_file_or_fallback("assets/glass.png", Color::new(200, 230, 255, 100))
    );
    let torch = Material::new(MaterialType::Torch);
    let portal_mat = Material::with_texture(MaterialType::Portal, portal_tex);
    let sand = Material::with_texture(
        MaterialType::Sand,
        ImageTexture::from_file_or_fallback("assets/sand.png", Color::new(237, 201, 175, 255))
    );
    
    // === Suelo (base de grass y dirt) ===
    for x in -5..=5 {
        for z in -5..=5 {
            let px = x as f32 * 2.0;
            let pz = z as f32 * 2.0;
            scene.add_cube(
                Vector3::new(px, -1.0, pz),
                2.0,
                grass.clone(),
                grass_side.clone(),
                dirt.clone()
            );
        }
    }
    
    // === Casa de piedra ===
    // Paredes
    for y in 0..3 {
        let py = y as f32 * 2.0;
        // Pared frontal
        scene.add_cube(Vector3::new(-4.0, py, -4.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(-2.0, py, -4.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        // Puerta (vidrio)
        if y > 0 {
            scene.add_cube(Vector3::new(0.0, py, -4.0), 2.0, glass.clone(), glass.clone(), glass.clone());
        }
        scene.add_cube(Vector3::new(2.0, py, -4.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(4.0, py, -4.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        
        // Pared trasera
        scene.add_cube(Vector3::new(-4.0, py, 4.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(-2.0, py, 4.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(0.0, py, 4.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(2.0, py, 4.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(4.0, py, 4.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        
        // Pared izquierda
        scene.add_cube(Vector3::new(-4.0, py, -2.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(-4.0, py, 0.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(-4.0, py, 2.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        
        // Pared derecha
        scene.add_cube(Vector3::new(4.0, py, -2.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(4.0, py, 0.0), 2.0, stone.clone(), stone.clone(), stone.clone());
        scene.add_cube(Vector3::new(4.0, py, 2.0), 2.0, stone.clone(), stone.clone(), stone.clone());
    }
    
    // Techo
    for x in -2..=2 {
        for z in -2..=2 {
            let px = x as f32 * 2.0;
            let pz = z as f32 * 2.0;
            scene.add_cube(Vector3::new(px, 6.0, pz), 2.0, stone.clone(), stone.clone(), stone.clone());
        }
    }
    
    // === Antorchas alrededor de la casa ===
    scene.spheres.push(Sphere::new(Vector3::new(-6.0, 1.5, -6.0), 0.3, torch.clone()));
    scene.spheres.push(Sphere::new(Vector3::new(6.0, 1.5, -6.0), 0.3, torch.clone()));
    scene.spheres.push(Sphere::new(Vector3::new(-6.0, 1.5, 6.0), 0.3, torch.clone()));
    scene.spheres.push(Sphere::new(Vector3::new(6.0, 1.5, 6.0), 0.3, torch.clone()));
    
    // === Piscina de agua ===
    for x in 6..=8 {
        for z in -2..=0 {
            let px = x as f32 * 2.0;
            let pz = z as f32 * 2.0;
            // Borde de piedra
            if x == 6 || x == 8 || z == -2 || z == 0 {
                scene.add_cube(Vector3::new(px, 0.0, pz), 2.0, stone.clone(), stone.clone(), stone.clone());
            } else {
                // Agua dentro
                scene.add_cube(Vector3::new(px, 0.0, pz), 2.0, water.clone(), water.clone(), water.clone());
            }
        }
    }
    
    // === Portal (efecto especial) ===
    // Marco de obsidiana (simulado con piedra oscura)
    let obsidian = Material::with_texture(
        MaterialType::Stone,
        ImageTexture::from_file_or_fallback("assets/obsidian.png", Color::new(20, 10, 30, 255))
    );
    
    // Marco vertical
    for y in 0..4 {
        let py = y as f32 * 2.0;
        scene.add_cube(Vector3::new(-8.0, py, 8.0), 2.0, obsidian.clone(), obsidian.clone(), obsidian.clone());
        scene.add_cube(Vector3::new(-4.0, py, 8.0), 2.0, obsidian.clone(), obsidian.clone(), obsidian.clone());
    }
    // Marco horizontal
    scene.add_cube(Vector3::new(-6.0, 8.0, 8.0), 2.0, obsidian.clone(), obsidian.clone(), obsidian.clone());
    scene.add_cube(Vector3::new(-6.0, -2.0, 8.0), 2.0, obsidian.clone(), obsidian.clone(), obsidian.clone());
    
    // Interior del portal (efecto portal)
    for y in 1..4 {
        let py = y as f32 * 2.0;
        scene.add_cube(Vector3::new(-6.0, py, 8.0), 2.0, portal_mat.clone(), portal_mat.clone(), portal_mat.clone());
    }
    
    // === Playa de arena ===
    for x in -8..=-6 {
        for z in -2..=0 {
            let px = x as f32 * 2.0;
            let pz = z as f32 * 2.0;
            scene.add_cube(Vector3::new(px, 0.0, pz), 2.0, sand.clone(), sand.clone(), sand.clone());
        }
    }
    
    // === Torre de vidrio ===
    for y in 0..5 {
        let py = y as f32 * 2.0;
        scene.add_cube(Vector3::new(8.0, py, 8.0), 2.0, glass.clone(), glass.clone(), glass.clone());
    }
    
    // === Árbol decorativo (simulado con cubos) ===
    // Tronco
    let wood = Material::with_texture(
        MaterialType::Dirt,
        ImageTexture::from_file_or_fallback("assets/wood.png", Color::new(139, 90, 43, 255))
    );
    scene.add_cube(Vector3::new(-10.0, 0.0, 0.0), 2.0, wood.clone(), wood.clone(), wood.clone());
    scene.add_cube(Vector3::new(-10.0, 2.0, 0.0), 2.0, wood.clone(), wood.clone(), wood.clone());
    scene.add_cube(Vector3::new(-10.0, 4.0, 0.0), 2.0, wood.clone(), wood.clone(), wood.clone());
    
    // Hojas
    let leaves = Material::with_texture(
        MaterialType::Grass,
        ImageTexture::from_file_or_fallback("assets/leaves.png", Color::new(50, 150, 50, 255))
    );
    for x in -1..=1 {
        for z in -1..=1 {
            if x == 0 && z == 0 { continue; }
            scene.add_cube(
                Vector3::new(-10.0 + x as f32 * 2.0, 6.0, z as f32 * 2.0),
                2.0,
                leaves.clone(),
                leaves.clone(),
                leaves.clone()
            );
        }
    }
    scene.add_cube(Vector3::new(-10.0, 8.0, 0.0), 2.0, leaves.clone(), leaves.clone(), leaves.clone());
    
    // === Sol (esfera emisiva en el cielo) ===
    let sun_angle = time * 0.3;
    let sun_height = sun_angle.sin() * 20.0 + 15.0;
    let sun_forward = sun_angle.cos() * 30.0;
    let sun_mat = Material::new(MaterialType::Torch);
    scene.spheres.push(Sphere::new(
        Vector3::new(sun_forward, sun_height, -20.0),
        3.0,
        sun_mat
    ));
    
    scene
}
