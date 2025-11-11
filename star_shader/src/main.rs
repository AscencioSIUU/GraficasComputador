use raylib::prelude::*;
mod obj;
mod shader;

use obj::Obj;
use raylib::math::{Vector2, Vector3};
use shader::Vector3Ext;
use shader::{Fragment, Uniforms, star_shader, vertex_displacement};

fn rotate_y(v: Vector3, angle: f32) -> Vector3 {
    let c = angle.cos();
    let s = angle.sin();
    Vector3::new(v.x * c - v.z * s, v.y, v.x * s + v.z * c)
}

fn rotate_x(v: Vector3, angle: f32) -> Vector3 {
    let c = angle.cos();
    let s = angle.sin();
    Vector3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

fn perspective_project(v: Vector3, width: f32, height: f32, fov: f32) -> Option<Vector2> {
    let near = 0.1;
    if v.z <= near { return None; }
    let aspect = width / height;
    let f = 1.0 / (fov / 2.0).to_radians().tan();

    let ndc_x = (v.x * f / aspect) / v.z;
    let ndc_y = (v.y * f) / v.z;

    Some(Vector2::new(
        (ndc_x + 1.0) * 0.5 * width,
        (1.0 - ndc_y) * 0.5 * height,
    ))
}

fn main() {
    let (width, height) = (1400, 1000);
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Star Shader Lab - Animated Sun with Perlin Noise")
        .build();

    rl.set_target_fps(60);

    // Cargar esfera
    let mut vertex_array: Vec<Vector3> = Vec::new();
    let mut normals: Vec<Vector3> = Vec::new();
    let mut loaded = false;

    let try_paths = vec![
        "assets/sphere.obj", 
        "sphere.obj", 
        "../planet_shaders/assets/sphere.obj",
        "../static_shaders/sphere.obj"
    ];
    
    for p in try_paths {
        match Obj::load(p) {
            Ok(m) => {
                vertex_array = m.get_vertex_array();
                
                // Calcular normales
                for i in (0..vertex_array.len()).step_by(3) {
                    if i + 2 < vertex_array.len() {
                        let v0 = vertex_array[i];
                        let v1 = vertex_array[i + 1];
                        let v2 = vertex_array[i + 2];
                        
                        let center = Vector3::new(
                            (v0.x + v1.x + v2.x) / 3.0,
                            (v0.y + v1.y + v2.y) / 3.0,
                            (v0.z + v1.z + v2.z) / 3.0,
                        );
                        let len = (center.x * center.x + center.y * center.y + center.z * center.z).sqrt();
                        let normal = if len > 0.001 {
                            Vector3::new(center.x / len, center.y / len, center.z / len)
                        } else {
                            Vector3::new(0.0, 1.0, 0.0)
                        };
                        
                        normals.push(normal);
                        normals.push(normal);
                        normals.push(normal);
                    }
                }
                
                loaded = true;
                println!("✅ Esfera cargada desde: {}", p);
                break;
            }
            Err(_) => continue,
        }
    }

    if !loaded {
        eprintln!("❌ ERROR: No se pudo cargar sphere.obj");
        eprintln!("💡 Coloca sphere.obj en assets/ o en el directorio actual");
        return;
    }

    // Camera variables
    let mut cam_distance: f32 = 3.0;
    let mut cam_yaw: f32 = 1.57; // ~90 grados (vista lateral horizontal)
    let mut cam_pitch: f32 = 0.0; // Completamente horizontal
    let rot_speed: f32 = 0.02;
    let zoom_speed: f32 = 0.1;

    // (NEW) keep a simple camera looking +Z from -Z
    fn camera_position(dist: f32) -> Vector3 {
        Vector3::new(0.0, 0.0, -dist)
    }
    
    // Rotación automática de la estrella
    let mut star_rotation: f32 = 0.0;
    let mut auto_rotate = true;
    
    // Parámetros del shader (ajustables)
    let mut intensity: f32 = 1.0;
    let mut temperature: f32 = 0.5; // 0.0 = roja, 0.5 = amarilla, 1.0 = azul
    let mut enable_vertex_displacement = true;
    
    let mut time: f32 = 0.0;
    let fov: f32 = 60.0;

    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║           🌟 STAR SHADER LAB - CONTROLES 🌟             ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║ VISTA: Plano horizontal - Se ve el movimiento lateral    ║");
    println!("║                                                           ║");
    println!("║ CÁMARA:                                                   ║");
    println!("║   ← →        : Rotar horizontal                           ║");
    println!("║   ↑ ↓        : Rotar vertical                             ║");
    println!("║   + -        : Zoom in/out                                ║");
    println!("║   ESPACIO    : Toggle rotación automática                 ║");
    println!("║                                                           ║");
    println!("║ PARÁMETROS DE LA ESTRELLA:                                ║");
    println!("║   I / K      : Aumentar/Disminuir intensidad              ║");
    println!("║   T / G      : Aumentar/Disminuir temperatura             ║");
    println!("║              (Roja → Amarilla → Azul)                     ║");
    println!("║   V          : Toggle vertex displacement (corona)        ║");
    println!("║                                                           ║");
    println!("║ PRESETS:                                                  ║");
    println!("║   1          : Sol amarillo (nuestro Sol)                 ║");
    println!("║   2          : Gigante roja (Betelgeuse)                  ║");
    println!("║   3          : Estrella azul (Rigel)                      ║");
    println!("║                                                           ║");
    println!("║   ESC        : Salir                                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    println!("📍 Vista inicial: Plano horizontal para ver mejor el movimiento");

    while !rl.window_should_close() {
        time += rl.get_frame_time();
        
        // ========== CONTROLES DE CÁMARA ==========
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_RIGHT) { cam_yaw += rot_speed; }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT) { cam_yaw -= rot_speed; }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_UP) { 
            cam_pitch = (cam_pitch + rot_speed).min(1.5); 
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_DOWN) { 
            cam_pitch = (cam_pitch - rot_speed).max(-1.5); 
        }
        
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_KP_ADD) || 
           rl.is_key_down(raylib::consts::KeyboardKey::KEY_EQUAL) {
            cam_distance = (cam_distance - zoom_speed).max(1.5);
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_KP_SUBTRACT) || 
           rl.is_key_down(raylib::consts::KeyboardKey::KEY_MINUS) {
            cam_distance = (cam_distance + zoom_speed).min(10.0);
        }
        
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SPACE) {
            auto_rotate = !auto_rotate;
            println!("🔄 Rotación automática: {}", if auto_rotate { "ON" } else { "OFF" });
        }
        
        // ========== CONTROLES DE PARÁMETROS ==========
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_I) {
            intensity = (intensity + 0.02).min(2.0);
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_K) {
            intensity = (intensity - 0.02).max(0.1);
        }
        
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_T) {
            temperature = (temperature + 0.01).min(1.0);
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_G) {
            temperature = (temperature - 0.01).max(0.0);
        }
        
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_V) {
            enable_vertex_displacement = !enable_vertex_displacement;
            println!("✨ Vertex displacement: {}", if enable_vertex_displacement { "ON" } else { "OFF" });
        }
        
        // ========== PRESETS ==========
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ONE) {
            intensity = 1.0;
            temperature = 0.5;
            println!("☀️  Preset: Sol amarillo (tipo G)");
        }
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_TWO) {
            intensity = 1.2;
            temperature = 0.15;
            println!("🔴 Preset: Gigante roja (tipo M)");
        }
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_THREE) {
            intensity = 1.5;
            temperature = 0.9;
            println!("🔵 Preset: Estrella azul (tipo B)");
        }
        
        if auto_rotate {
            star_rotation += 0.003;
        }
        
        let cam_pos = camera_position(cam_distance);

        let fps = rl.get_fps();

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(2, 2, 8, 255)); // Espacio profundo

        // ========== UI ==========
        d.draw_text("🌟 ESTRELLA ANIMADA CON PERLIN NOISE", 20, 20, 28, Color::GOLD);
        
        let star_type = if temperature < 0.3 { "Gigante Roja" } 
                       else if temperature < 0.6 { "Sol Amarillo" } 
                       else { "Estrella Azul" };
        d.draw_text(&format!("Tipo: {} | Temp: {:.2} | Intensidad: {:.2}", 
            star_type, temperature, intensity), 20, 55, 20, Color::WHITE);
        
        d.draw_text(&format!("Vertex Displacement: {}", 
            if enable_vertex_displacement { "ON ✓" } else { "OFF" }), 20, 80, 18, Color::LIGHTGRAY);
        
        d.draw_text("I/K: Intensidad | T/G: Temperatura | V: Corona | 1/2/3: Presets", 
            20, 105, 16, Color::DARKGRAY);
        
        d.draw_text(&format!("FPS: {}", fps), width - 120, 20, 24, Color::LIME);

        // ========== RENDERIZAR ESTRELLA ==========
        let uniforms = Uniforms::new(time, intensity, temperature);

        for i in (0..vertex_array.len()).step_by(3) {
            if i + 2 >= vertex_array.len() { break; }
            
            // === PER TRIANGLE ===
        let mut v0 = vertex_array[i];
        let mut v1 = vertex_array[i + 1];
        let mut v2 = vertex_array[i + 2];
        let mut n0 = normals[i];     // per-face normal replicated 3x
        let mut n1 = normals[i + 1];
        let mut n2 = normals[i + 2];

        // 1) Optional surface displacement
        if enable_vertex_displacement {
            v0 = vertex_displacement(v0, time);
            v1 = vertex_displacement(v1, time);
            v2 = vertex_displacement(v2, time);
        }

        // 2) Star/object rotation
        v0 = rotate_y(v0, star_rotation);
        v1 = rotate_y(v1, star_rotation);
        v2 = rotate_y(v2, star_rotation);
        n0 = rotate_y(n0, star_rotation);
        n1 = rotate_y(n1, star_rotation);
        n2 = rotate_y(n2, star_rotation);

        // 3) "Orbit" by rotating the MODEL with camera yaw/pitch
        //    (camera stays fixed at (0,0,-dist) looking +Z)
        v0 = rotate_x(v0, cam_pitch);
        v1 = rotate_x(v1, cam_pitch);
        v2 = rotate_x(v2, cam_pitch);
        n0 = rotate_x(n0, cam_pitch);
        n1 = rotate_x(n1, cam_pitch);
        n2 = rotate_x(n2, cam_pitch);

        v0 = rotate_y(v0, cam_yaw);
        v1 = rotate_y(v1, cam_yaw);
        v2 = rotate_y(v2, cam_yaw);
        n0 = rotate_y(n0, cam_yaw);
        n1 = rotate_y(n1, cam_yaw);
        n2 = rotate_y(n2, cam_yaw);

        // 4) View transform: translate only (camera is fixed)
        let mut c0 = Vector3::new(v0.x - cam_pos.x, v0.y - cam_pos.y, v0.z - cam_pos.z);
        let mut c1 = Vector3::new(v1.x - cam_pos.x, v1.y - cam_pos.y, v1.z - cam_pos.z);
        let mut c2 = Vector3::new(v2.x - cam_pos.x, v2.y - cam_pos.y, v2.z - cam_pos.z);

        // 5) Backface culling — flip condition to keep front faces for your winding
        let edge1 = Vector3::new(c1.x - c0.x, c1.y - c0.y, c1.z - c0.z);
        let edge2 = Vector3::new(c2.x - c0.x, c2.y - c0.y, c2.z - c0.z);
        let normal = Vector3::new(
            edge1.y * edge2.z - edge1.z * edge2.y,
            edge1.z * edge2.x - edge1.x * edge2.z,
            edge1.x * edge2.y - edge1.y * edge2.x,
        );
        let center = Vector3::new(
            (c0.x + c1.x + c2.x) / 3.0,
            (c0.y + c1.y + c2.y) / 3.0,
            (c0.z + c1.z + c2.z) / 3.0,
        );

        // camera is at origin after view transform, so view_dir points from triangle to camera
        let view_dir = Vector3::new(-center.x, -center.y, -center.z);
        let dot = normal.x * view_dir.x + normal.y * view_dir.y + normal.z * view_dir.z;

        // FLIP: discard only when facing away under this winding
        if dot >= 0.0 { continue; }

        // 6) Project (now z should be positive)
        let p0 = perspective_project(c0, width as f32, height as f32, fov);
        let p1 = perspective_project(c1, width as f32, height as f32, fov);
        let p2 = perspective_project(c2, width as f32, height as f32, fov);
        if p0.is_none() || p1.is_none() || p2.is_none() { continue; }
        let p0 = p0.unwrap();
        let p1 = p1.unwrap();
        let p2 = p2.unwrap();

        // Fragment shading: use model-space center but with same rotations applied
        let world_center = Vector3::new(
            (v0.x + v1.x + v2.x) / 3.0,
            (v0.y + v1.y + v2.y) / 3.0,
            (v0.z + v1.z + v2.z) / 3.0,
        );

        // Pick any rotated normal; averaging is optional
        let n_rot = Vector3::new(
            (n0.x + n1.x + n2.x) / 3.0,
            (n0.y + n1.y + n2.y) / 3.0,
            (n0.z + n1.z + n2.z) / 3.0,
        );

        let fragment = Fragment {
            world_position: world_center,
            normal: n_rot.normalized(),
        };

        let color = star_shader(&fragment, &uniforms);
        let fill_color = Color::new(
            (color.x * 255.0).clamp(0.0, 255.0) as u8,
            (color.y * 255.0).clamp(0.0, 255.0) as u8,
            (color.z * 255.0).clamp(0.0, 255.0) as u8,
            255,
        );

        d.draw_triangle(p0, p1, p2, fill_color);

        }
    }
    
    println!("\n👋 ¡Hasta luego! Gracias por usar Star Shader Lab");
}
