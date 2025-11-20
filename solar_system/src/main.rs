use raylib::prelude::*;
mod obj;
mod shaders;
mod controls; // 👈 changed from `mod ship;`

use obj::Obj;
use raylib::math::Vector3;
use shaders::{
    Fragment, Uniforms,
    star_shader,
    vertex_displacement,
    cellular_planet_shader,
    vertex_displacement_mercury,
    simplex_planet_shader,
    vertex_displacement_venus,
    voronoi_planet_shader,
    vertex_displacement_earth,
    perlin_planet_shader,
    vertex_displacement_mars,
    planet_shader,
    vertex_displacement_goliath,
    spaceship_shader,
};
use controls::ShipControls; // 👈 use ShipControls instead of Ship

// Para generar ángulos aleatorios de las estrellas
struct StarLine {
    angle: f32,
    speed: f32,
}

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

fn rotate_z(v: Vector3, angle: f32) -> Vector3 {
    let c = angle.cos();
    let s = angle.sin();
    Vector3::new(v.x * c - v.y * s, v.x * s + v.y * c, v.z)
}

fn perspective_project(v: Vector3, width: f32, height: f32, fov: f32) -> Option<(f32,f32)> {
    let near = 0.05;
    if v.z <= near { return None; }
    let aspect = width / height;
    let f = 1.0 / (fov / 2.0).to_radians().tan();

    let ndc_x = (v.x * f / aspect) / v.z;
    let ndc_y = (v.y * f) / v.z;

    Some(((ndc_x + 1.0) * 0.5 * width, (1.0 - ndc_y) * 0.5 * height))
}

struct Body {
    name: &'static str,
    orbit_radius: f32,
    orbit_speed: f32,
    size: f32,
    color: Vector3,
    spin_speed: f32,
}

struct TriangleDepth {
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    color: Color,
    depth: f32,
}

fn main() {
    let (width, height) = (1280, 800);
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Solar System - Software Renderer")
        .build();

    rl.set_target_fps(60);

    // ---------------------------
    // Cargar esfera con más detalle
    // ---------------------------
    let mut sphere_tris: Vec<Vector3> = Vec::new();
    let sphere_paths = [
        "sphere.obj",
        "assets/sphere.obj",
        "../planet_shaders/assets/sphere.obj",
        "../star_shader/assets/sphere.obj",
    ];
    let mut loaded = false;
    for p in sphere_paths {
        if let Ok(m) = Obj::load(p) {
            sphere_tris = m.get_vertex_array();
            println!("✓ Loaded sphere from {} (tris={})", p, sphere_tris.len()/3);
            loaded = true; break;
        }
    }
    if !loaded { eprintln!("Could not load sphere.obj — place it in assets/"); return; }
    
    // Aumentar detalle: +50% más triángulos (1500 → 2250)
    let max_tris = 2250;
    if sphere_tris.len() / 3 > max_tris {
        println!("⚡ Optimizing: reducing {} → {} tris", sphere_tris.len()/3, max_tris);
        sphere_tris.truncate(max_tris * 3);
    }

    // ---------------------------
    // Cargar nave
    // ---------------------------
    let mut ship_tris: Vec<Vector3> = Vec::new();
    let ship_paths = [
        "spaceship.obj",
        "../spaceship/spaceship.mtl.obj",
        "spaceship.mtl.obj",
        "../spaceship/spaceship.obj",
    ];
    let mut ship_loaded = false;
    for p in ship_paths {
        if let Ok(m) = Obj::load(p) {
            ship_tris = m.get_vertex_array();
            println!("✓ Loaded spaceship from {} (tris={})", p, ship_tris.len()/3);
            ship_loaded = true;
            break;
        }
    }
    if !ship_loaded {
        eprintln!("⚠ No spaceship model found — using fallback marker");
    }

    // ---------------------------
    // Planetas
    // ---------------------------
    let bodies = vec![
        Body { name: "Sun", orbit_radius: 0.0, orbit_speed: 0.0, size: 18.0, color: Vector3::new(1.0,0.9,0.6), spin_speed: 0.01 },
        Body { name: "Mercury", orbit_radius: 20.0, orbit_speed: 0.6, size: 3.6, color: Vector3::new(0.6,0.6,0.65), spin_speed: 0.15 },
        Body { name: "Venus", orbit_radius: 30.0, orbit_speed: 0.4, size: 5.4, color: Vector3::new(0.9,0.7,0.4), spin_speed: 0.05 },
        Body { name: "Earth", orbit_radius: 42.0, orbit_speed: 0.3, size: 6.0, color: Vector3::new(0.2,0.5,0.9), spin_speed: 0.3 },
        Body { name: "Mars", orbit_radius: 55.0, orbit_speed: 0.225, size: 4.5, color: Vector3::new(0.9,0.4,0.25), spin_speed: 0.25 },
        Body { name: "Goliath", orbit_radius: 70.0, orbit_speed: 0.125, size: 13.2, color: Vector3::new(0.5,0.2,0.9), spin_speed: 0.025 },
    ];

    // ---------------------------
    // Nave + cámara
    // ---------------------------
    let mut ship = ShipControls::new(Vector3 { x: 0.0, y: 0.0, z: -25.0 });
    let ship_radius = 0.35f32;

    // view mode: 1 = top, 2 = side, 3 = third-person
    let mut view_mode: i32 = 3;
    let mut free_camera_mode: bool = false;
    
    // Free camera (vista fija del sistema)
    let mut free_cam_height: f32 = 80.0;
    let mut free_cam_distance: f32 = 50.0;
    let mut free_cam_angle: f32 = 0.0;
    
    // Sistema de warp/teletransporte
    let mut warp_active = false;
    let mut warp_progress: f32 = 0.0;
    let mut warp_start_pos = Vector3::new(0.0, 0.0, 0.0);
    let mut warp_target_pos = Vector3::new(0.0, 0.0, 0.0);
    let warp_duration: f32 = 1.5; // segundos
    
    let mut paused: bool = false;
    let mut time: f32 = 0.0;           // Para animaciones (siempre avanza)
    let mut orbit_time: f32 = 0.0;     // Para órbitas (se congela con pause)
    
    // Generar líneas de estrellas aleatorias para el efecto warp (estilo Star Wars)
    let mut star_lines: Vec<StarLine> = Vec::new();
    for i in 0..200 {
        star_lines.push(StarLine {
            angle: (i as f32 * 13.7 + i as f32 * i as f32 * 0.3) % 360.0,
            speed: 1.0 + ((i * 17) % 100) as f32 / 100.0,
        });
    }

    println!("Controls: W/S/A/D move, Space/Ctrl up/down, Arrows rotate, X nitro");
    println!("V: FREE CAMERA, T: Pause");

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        // -----------------------
        // Input global
        // -----------------------
        if rl.is_key_pressed(KeyboardKey::KEY_T) {
            paused = !paused;
            println!("⏸ Paused: {}", paused);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_V) {
            free_camera_mode = !free_camera_mode;
            if free_camera_mode {
                println!("📷 FREE CAMERA MODE: Fixed elevated view of solar system");
                free_cam_angle = 0.0;
                free_cam_distance = 50.0;
                free_cam_height = 100.0;
            } else {
                println!("🚀 EXPLORATION MODE: Controls enabled");
                ship.position.y = 0.0;
                ship.yaw = 0.0;
                ship.pitch = 0.0;
            }
        }

        // Time always advances for animations (planet spin, shaders)
        time += dt;
        
        // Orbit time only advances when not paused (freezes planet positions)
        if !paused {
            orbit_time += dt;
        }

        // -----------------------
        // Sistema de Teletransporte con Animación
        // -----------------------
        if !free_camera_mode && !warp_active {
            if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_ONE) {
                let angle = orbit_time * bodies[0].orbit_speed;
                warp_start_pos = ship.position;
                warp_target_pos = Vector3::new(
                    angle.cos() * bodies[0].orbit_radius,
                    0.0,
                    angle.sin() * bodies[0].orbit_radius - bodies[0].size - 15.0
                );
                warp_active = true;
                warp_progress = 0.0;
                println!("⚡ Iniciando warp al Sol");
            }
            else if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_TWO) {
                let angle = orbit_time * bodies[1].orbit_speed;
                warp_start_pos = ship.position;
                warp_target_pos = Vector3::new(
                    angle.cos() * bodies[1].orbit_radius,
                    0.0,
                    angle.sin() * bodies[1].orbit_radius - bodies[1].size - 10.0
                );
                warp_active = true;
                warp_progress = 0.0;
                println!("⚡ Iniciando warp a Mercurio");
            }
            else if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_THREE) {
                let angle = orbit_time * bodies[2].orbit_speed;
                warp_start_pos = ship.position;
                warp_target_pos = Vector3::new(
                    angle.cos() * bodies[2].orbit_radius,
                    0.0,
                    angle.sin() * bodies[2].orbit_radius - bodies[2].size - 10.0
                );
                warp_active = true;
                warp_progress = 0.0;
                println!("⚡ Iniciando warp a Venus");
            }
            else if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_FOUR) {
                let angle = orbit_time * bodies[3].orbit_speed;
                warp_start_pos = ship.position;
                warp_target_pos = Vector3::new(
                    angle.cos() * bodies[3].orbit_radius,
                    0.0,
                    angle.sin() * bodies[3].orbit_radius - bodies[3].size - 10.0
                );
                warp_active = true;
                warp_progress = 0.0;
                println!("⚡ Iniciando warp a la Tierra");
            }
            else if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_FIVE) {
                let angle = orbit_time * bodies[4].orbit_speed;
                warp_start_pos = ship.position;
                warp_target_pos = Vector3::new(
                    angle.cos() * bodies[4].orbit_radius,
                    0.0,
                    angle.sin() * bodies[4].orbit_radius - bodies[4].size - 10.0
                );
                warp_active = true;
                warp_progress = 0.0;
                println!("⚡ Iniciando warp a Marte");
            }
            else if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SIX) {
                let angle = orbit_time * bodies[5].orbit_speed;
                warp_start_pos = ship.position;
                warp_target_pos = Vector3::new(
                    angle.cos() * bodies[5].orbit_radius,
                    0.0,
                    angle.sin() * bodies[5].orbit_radius - bodies[5].size - 15.0
                );
                warp_active = true;
                warp_progress = 0.0;
                println!("⚡ Iniciando warp a Goliath");
            }
        }
        
        // Actualizar animación de warp
        if warp_active {
            warp_progress += dt / warp_duration;
            
            if warp_progress >= 1.0 {
                // Warp completado
                warp_progress = 1.0;
                warp_active = false;
                ship.position = warp_target_pos;
                ship.velocity = Vector3::new(0.0, 0.0, 0.0);
                println!("✅ Warp completado!");
            } else {
                // Interpolación suave con easing
                let t = warp_progress;
                // Easing: aceleración al inicio, desaceleración al final
                let eased_t = if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                };
                
                // Interpolar posición
                ship.position.x = warp_start_pos.x + (warp_target_pos.x - warp_start_pos.x) * eased_t;
                ship.position.y = warp_start_pos.y + (warp_target_pos.y - warp_start_pos.y) * eased_t;
                ship.position.z = warp_start_pos.z + (warp_target_pos.z - warp_start_pos.z) * eased_t;
                
                // Resetear velocidad durante el warp
                ship.velocity = Vector3::new(0.0, 0.0, 0.0);
            }
        }

        // -----------------------
        // Actualizar nave/cámara (solo si no está en warp)
        // -----------------------
        if !free_camera_mode && !paused && !warp_active {
            let old_pos = ship.position;
            
            // Actualizar movimiento
            ship.update(&rl, 0.003, 0.05);

            // Colisiones con planetas - prevenir atravesar
            for b in &bodies {
                let angle = orbit_time * b.orbit_speed;
                let planet_x = angle.cos() * b.orbit_radius;
                let planet_z = angle.sin() * b.orbit_radius;
                let planet_y = 0.0;

                let dx = ship.position.x - planet_x;
                let dy = ship.position.y - planet_y;
                let dz = ship.position.z - planet_z;
                let dist = (dx*dx + dy*dy + dz*dz).sqrt();
                let collision_radius = b.size;

                if dist < collision_radius {
                    ship.position = old_pos;
                    ship.velocity = Vector3::new(0.0, 0.0, 0.0);
                    break;
                }
            }
        }

        // -----------------------
        // Posición de la cámara
        // -----------------------
        let cam_world_x: f32;
        let cam_world_z: f32;
        let cam_y: f32;
        let cam_yaw: f32;
        let cam_pitch: f32;

        if free_camera_mode {
            cam_world_x = free_cam_angle.sin() * free_cam_distance;
            cam_world_z = free_cam_angle.cos() * free_cam_distance;
            cam_y = free_cam_height;

            // Mirar al centro del sistema
            cam_yaw = free_cam_angle + std::f32::consts::PI;
            cam_pitch = -(free_cam_height / (free_cam_height.powi(2) + free_cam_distance.powi(2)).sqrt()).asin();
        } else {
            match view_mode {
                1 => {
                    // top-down
                    cam_world_x = ship.position.x;
                    cam_world_z = ship.position.z;
                    cam_y = ship.position.y + 18.0;
                    cam_yaw = ship.yaw;
                    cam_pitch = ship.pitch;
                }
                2 => {
                    // side
                    cam_world_x = ship.position.x - 12.0;
                    cam_world_z = ship.position.z;
                    cam_y = ship.position.y + 4.0;
                    cam_yaw = ship.yaw;
                    cam_pitch = ship.pitch;
                }
                _ => {
                    // third-person detrás de la nave
                    let behind_dist = 3.0;
                    cam_world_x = ship.position.x - ship.yaw.sin() * behind_dist;
                    cam_world_z = ship.position.z - ship.yaw.cos() * behind_dist;
                    cam_y = ship.position.y + 1.5;
                    cam_yaw = ship.yaw;
                    cam_pitch = ship.pitch;
                }
            }
        }

        // -----------------------
        // DRAW
        // -----------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(4,4,16,255));
        d.draw_text("Solar System - Software Renderer", 20, 20, 24, Color::LIGHTGRAY);

        let mode_indicator = if free_camera_mode { 
            "📷 FREE CAMERA - Elevated View" 
        } else { 
            "🚀 EXPLORATION" 
        };
        let status = if paused { "⏸ PAUSED" } else { "▶ Active" };

        if free_camera_mode {
            d.draw_text(
                &format!(
                    "Height: {:.1} | Distance: {:.1} | Angle: {:.1}°  {} {}",
                    free_cam_height, free_cam_distance, free_cam_angle.to_degrees(), mode_indicator, status
                ),
                20, 50, 18, Color::WHITE
            );
            d.draw_text("Fixed Camera View | V: Exit Free Cam | T: Pause", 20, 74, 16, Color::DARKGRAY);
        } else {
            d.draw_text(
                &format!(
                    "Ship: ({:.2},{:.2})  Yaw:{:.1}°  {} {}",
                    ship.position.x, ship.position.z,
                    cam_yaw.to_degrees(),
                    mode_indicator, status
                ),
                20, 50, 18, Color::WHITE
            );
            d.draw_text("W/S/A/D: move | Arrows: rotate | V: FREE CAMERA | T: pause", 20, 74, 16, Color::DARKGRAY);
            
            // Panel de teletransporte más visible
            d.draw_rectangle(width - 260, 10, 250, 160, Color::new(0, 0, 0, 180));
            d.draw_rectangle_lines(width - 260, 10, 250, 160, Color::new(100, 200, 255, 255));
            d.draw_text("⚡ INSTANT WARP", width - 250, 20, 20, Color::new(100, 200, 255, 255));
            d.draw_text("1 - ☀️  Sun", width - 240, 50, 18, Color::new(255, 220, 100, 255));
            d.draw_text("2 - 🪐 Mercury", width - 240, 72, 18, Color::new(180, 180, 190, 255));
            d.draw_text("3 - 🌕 Venus", width - 240, 94, 18, Color::new(230, 180, 100, 255));
            d.draw_text("4 - 🌍 Earth", width - 240, 116, 18, Color::new(80, 150, 230, 255));
            d.draw_text("5 - 🔴 Mars", width - 240, 138, 18, Color::new(230, 100, 60, 255));
            d.draw_text("6 - 💜 Goliath", width - 240, 160, 18, Color::new(150, 80, 200, 255));
        }

        // Estrellas de fondo
        for i in 0..120 {
            let sx = ((i * 97) % width) as f32 + ((time * 10.0).sin() * 2.0) as f32;
            let sy = ((i * 53) % height) as f32 + ((time * 7.0).cos() * 2.0) as f32;
            d.draw_pixel(sx as i32, sy as i32, Color::new(200,200,255, 200));
        }

        // Órbitas
        for (idx, b) in bodies.iter().enumerate() {
            if idx == 0 { continue; }
            let orbit_points = 100;
            for i in 0..orbit_points {
                let angle1 = (i as f32 / orbit_points as f32) * std::f32::consts::PI * 2.0;
                let angle2 = ((i + 1) as f32 / orbit_points as f32) * std::f32::consts::PI * 2.0;
                
                let ox1 = angle1.cos() * b.orbit_radius;
                let oz1 = angle1.sin() * b.orbit_radius;
                let ox2 = angle2.cos() * b.orbit_radius;
                let oz2 = angle2.sin() * b.orbit_radius;
                
                let mut c1 = Vector3::new(ox1 - cam_world_x, 0.0 - cam_y, oz1 - cam_world_z);
                let mut c2 = Vector3::new(ox2 - cam_world_x, 0.0 - cam_y, oz2 - cam_world_z);
                
                c1 = rotate_x(c1, -cam_pitch);
                c1 = rotate_y(c1, -cam_yaw);
                c2 = rotate_x(c2, -cam_pitch);
                c2 = rotate_y(c2, -cam_yaw);
                
                if let Some((x1, y1)) = perspective_project(c1, width as f32, height as f32, 70.0) {
                    if let Some((x2, y2)) = perspective_project(c2, width as f32, height as f32, 70.0) {
                        let orbit_color = Color::new(
                            (b.color.x * 80.0) as u8,
                            (b.color.y * 80.0) as u8,
                            (b.color.z * 80.0) as u8,
                            120
                        );
                        d.draw_line_v(Vector2::new(x1, y1), Vector2::new(x2, y2), orbit_color);
                    }
                }
            }
        }

        // Triángulos de planetas
        let mut triangles: Vec<TriangleDepth> = Vec::new();

        for (idx, b) in bodies.iter().enumerate() {
            let angle = orbit_time * b.orbit_speed;
            let bx = angle.cos() * b.orbit_radius;
            let bz = angle.sin() * b.orbit_radius;

            for i in (0..sphere_tris.len()).step_by(3) {
                if i + 2 >= sphere_tris.len() { break; }
                let mut v0 = sphere_tris[i];
                let mut v1 = sphere_tris[i+1];
                let mut v2 = sphere_tris[i+2];

                v0 *= b.size;
                v1 *= b.size;
                v2 *= b.size;

                // Aplicar vertex displacement seg煤n el planeta
                match idx {
                    0 => {
                        // Sol
                        v0 = vertex_displacement(v0, time);
                        v1 = vertex_displacement(v1, time);
                        v2 = vertex_displacement(v2, time);
                    },
                    1 => {
                        // Mercury
                        v0 = vertex_displacement_mercury(v0, time);
                        v1 = vertex_displacement_mercury(v1, time);
                        v2 = vertex_displacement_mercury(v2, time);
                    },
                    2 => {
                        // Venus
                        v0 = vertex_displacement_venus(v0, time);
                        v1 = vertex_displacement_venus(v1, time);
                        v2 = vertex_displacement_venus(v2, time);
                    },
                    3 => {
                        // Earth
                        v0 = vertex_displacement_earth(v0, time);
                        v1 = vertex_displacement_earth(v1, time);
                        v2 = vertex_displacement_earth(v2, time);
                    },
                    4 => {
                        // Mars
                        v0 = vertex_displacement_mars(v0, time);
                        v1 = vertex_displacement_mars(v1, time);
                        v2 = vertex_displacement_mars(v2, time);
                    },
                    5 => {
                        // Goliath
                        v0 = vertex_displacement_goliath(v0, time);
                        v1 = vertex_displacement_goliath(v1, time);
                        v2 = vertex_displacement_goliath(v2, time);
                    },
                    _ => {}
                }

                let spin_angle = time * b.spin_speed;
                v0 = rotate_y(v0, spin_angle);
                v1 = rotate_y(v1, spin_angle);
                v2 = rotate_y(v2, spin_angle);

                v0.x += bx; v0.z += bz;
                v1.x += bx; v1.z += bz;
                v2.x += bx; v2.z += bz;

                let mut c0 = Vector3::new(v0.x - cam_world_x, v0.y - cam_y, v0.z - cam_world_z);
                let mut c1 = Vector3::new(v1.x - cam_world_x, v1.y - cam_y, v1.z - cam_world_z);
                let mut c2 = Vector3::new(v2.x - cam_world_x, v2.y - cam_y, v2.z - cam_world_z);

                c0 = rotate_x(c0, -cam_pitch);
                c1 = rotate_x(c1, -cam_pitch);
                c2 = rotate_x(c2, -cam_pitch);
                c0 = rotate_y(c0, -cam_yaw);
                c1 = rotate_y(c1, -cam_yaw);
                c2 = rotate_y(c2, -cam_yaw);

                // ==========================================
                // OPTIMIZACIÓN 1: Backface Culling
                // Descarta triángulos que miran hacia otro lado
                // Reduce el procesamiento ~50% (solo caras visibles)
                // ==========================================
                let edge1 = c1 - c0;
                let edge2 = c2 - c0;
                let normal = Vector3::new(
                    edge1.y * edge2.z - edge1.z * edge2.y,
                    edge1.z * edge2.x - edge1.x * edge2.z,
                    edge1.x * edge2.y - edge1.y * edge2.x,
                );
                let center = (c0 + c1 + c2) / 3.0;
                let view_dir = -center;
                let dot = normal.x * view_dir.x + normal.y * view_dir.y + normal.z * view_dir.z;
                if dot >= 0.0 { continue; } // Backface culling

                // ==========================================
                // OPTIMIZACIÓN 2: Early Rejection
                // Si algún vértice está detrás de la cámara, descarta
                // Evita cálculos de proyección innecesarios
                // ==========================================
                let p0 = perspective_project(c0, width as f32, height as f32, 70.0);
                let p1 = perspective_project(c1, width as f32, height as f32, 70.0);
                let p2 = perspective_project(c2, width as f32, height as f32, 70.0);
                if p0.is_none() || p1.is_none() || p2.is_none() { continue; } // Early rejection
                let (x0,y0) = p0.unwrap(); let (x1,y1) = p1.unwrap(); let (x2,y2) = p2.unwrap();

                // ==========================================
                // OPTIMIZACIÓN 3: Shader per-triangle
                // Calcula shader 1 vez por triángulo en lugar de per-pixel
                // Reduce de millones a ~1500 llamadas por planeta
                // ==========================================
                let len = (normal.x*normal.x + normal.y*normal.y + normal.z*normal.z).sqrt();
                let n = if len > 0.0001 {
                    normal / len
                } else {
                    Vector3::new(0.0,1.0,0.0)
                };

                let world_center = (v0 + v1 + v2) / 3.0;
                let fragment = Fragment { world_position: world_center, normal: n };

                let color_v = match idx {
                    0 => {
                        let uniforms = Uniforms::new(time, 1.0);
                        star_shader(&fragment, &uniforms)
                    },
                    1 => cellular_planet_shader(&fragment, time, b.color),
                    2 => simplex_planet_shader(&fragment, time, b.color),
                    3 => voronoi_planet_shader(&fragment, time, b.color),
                    4 => perlin_planet_shader(&fragment, time, b.color),
                    _ => b.color,
                };

                // ==========================================
                // OPTIMIZACIÓN 4: Depth Sorting
                // Ordena triángulos por profundidad antes de dibujar
                // Asegura renderizado correcto sin Z-buffer costoso
                // ==========================================
                let fill = Color::new(
                    (color_v.x*255.0) as u8,
                    (color_v.y*255.0) as u8,
                    (color_v.z*255.0) as u8,
                    255
                );
                
                let avg_depth = (c0.z + c1.z + c2.z) / 3.0;
                triangles.push(TriangleDepth {
                    p0: (x0, y0),
                    p1: (x1, y1),
                    p2: (x2, y2),
                    color: fill,
                    depth: avg_depth,
                });
            }
        }

        // ==========================================
        // OPTIMIZACIÓN 5: Sorting eficiente
        // Ordena de atrás hacia adelante para painter's algorithm
        // Solo se hace 1 vez por frame, no por triángulo
        // ==========================================
        triangles.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap());

        for tri in &triangles {
            d.draw_triangle(
                Vector2::new(tri.p0.0, tri.p0.1),
                Vector2::new(tri.p1.0, tri.p1.1),
                Vector2::new(tri.p2.0, tri.p2.1),
                tri.color
            );
        }

        // -----------------------
        // Nave (cockpit) – sólo en modo exploración
        // -----------------------
        if !ship_tris.is_empty() && !free_camera_mode {
            let ship_scale = 0.05f32;
            let ship_cam_x = 0.0;
            let ship_cam_y = -0.4;
            let ship_cam_z = 1.2;
            
            for i in (0..ship_tris.len()).step_by(3) {
                if i + 2 >= ship_tris.len() { break; }
                let mut a = ship_tris[i];
                let mut b_ = ship_tris[i+1];
                let mut c_ = ship_tris[i+2];

                a *= ship_scale;
                b_ *= ship_scale;
                c_ *= ship_scale;

                let rotation_y = -90.0_f32.to_radians();
                a = rotate_y(a, rotation_y);
                b_ = rotate_y(b_, rotation_y);
                c_ = rotate_y(c_, rotation_y);
                
                a.x += ship_cam_x; a.y += ship_cam_y; a.z += ship_cam_z;
                b_.x += ship_cam_x; b_.y += ship_cam_y; b_.z += ship_cam_z;
                c_.x += ship_cam_x; c_.y += ship_cam_y; c_.z += ship_cam_z;

                let ca = a;
                let cb = b_;
                let cc = c_;

                let e1 = cb - ca;
                let e2 = cc - ca;
                let n = Vector3::new(
                    e1.y * e2.z - e1.z * e2.y,
                    e1.z * e2.x - e1.x * e2.z,
                    e1.x * e2.y - e1.y * e2.x,
                );
                let center = (ca + cb + cc) / 3.0;
                let view_dir = -center;
                let dotp = n.x * view_dir.x + n.y * view_dir.y + n.z * view_dir.z;
                if dotp >= 0.0 { continue; }

                let pa = perspective_project(ca, width as f32, height as f32, 70.0);
                let pb = perspective_project(cb, width as f32, height as f32, 70.0);
                let pc = perspective_project(cc, width as f32, height as f32, 70.0);
                if pa.is_none() || pb.is_none() || pc.is_none() { continue; }
                let (ax,ay) = pa.unwrap(); let (bx,by) = pb.unwrap(); let (cx,cy) = pc.unwrap();
                
                let v0 = ship_tris[i];
                let v1 = ship_tris[i+1];
                let v2 = ship_tris[i+2];
                let world_center = (v0 + v1 + v2) / 3.0;
                
                let normal_len = (n.x*n.x + n.y*n.y + n.z*n.z).sqrt();
                let normal = if normal_len > 0.0001 {
                    n / normal_len
                } else {
                    Vector3::new(0.0, 1.0, 0.0)
                };
                
                let fragment = Fragment { world_position: world_center, normal };
                let shader_color = spaceship_shader(&fragment, time);
                
                let final_color = shader_color;
                
                let ship_color = Color::new(
                    (final_color.x * 255.0).clamp(0.0, 255.0) as u8,
                    (final_color.y * 255.0).clamp(0.0, 255.0) as u8,
                    (final_color.z * 255.0).clamp(0.0, 255.0) as u8,
                    255
                );
                
                d.draw_triangle(Vector2::new(ax,ay), Vector2::new(bx,by), Vector2::new(cx,cy), ship_color);
            }
        } else if !free_camera_mode {
            d.draw_circle((width/2) as i32, (height - 80) as i32, 12.0, Color::WHITE);
        }
        
        if free_camera_mode {
            d.draw_text(&format!("Camera: ({:.1}, {:.1}, {:.1})", cam_world_x, cam_y, cam_world_z), 20, height - 48, 18, Color::LIGHTGRAY);
        } else {
            d.draw_text(&format!("Ship pos ({:.2},{:.2})", ship.position.x, ship.position.z), 20, height - 48, 18, Color::LIGHTGRAY);
        }
        
        // -----------------------
        // Efecto de estrellas pasando (estilo Star Wars)
        // -----------------------
        if warp_active {
            for star in &star_lines {
                let speed = star.speed;
                let distance = (warp_progress * 2500.0 * speed) % 2500.0;
                
                // Dirección desde el centro
                let dir_x = star.angle.to_radians().cos();
                let dir_y = star.angle.to_radians().sin();
                
                // Punto inicial y final de la línea
                let start_x = (width / 2) as f32 + dir_x * distance;
                let start_y = (height / 2) as f32 + dir_y * distance;
                let end_x = (width / 2) as f32 + dir_x * (distance + 120.0 * speed);
                let end_y = (height / 2) as f32 + dir_y * (distance + 120.0 * speed);
                
                // Fade out cuando se alejan
                let alpha = if distance > 2000.0 {
                    ((2500.0 - distance) / 500.0 * 255.0) as u8
                } else if distance < 200.0 {
                    (distance / 200.0 * 255.0) as u8
                } else {
                    255
                };
                
                // Grosor variable según velocidad
                let thickness = 1.5 + speed * 0.5;
                
                d.draw_line_ex(
                    Vector2::new(start_x, start_y),
                    Vector2::new(end_x, end_y),
                    thickness,
                    Color::new(220, 235, 255, alpha)
                );
            }
            
            // Texto de warp
            let warp_percent = (warp_progress * 100.0) as i32;
            d.draw_text(
                &format!("WARP: {}%", warp_percent),
                (width / 2 - 80) as i32,
                50,
                32,
                Color::new(100, 200, 255, 255)
            );
        }
    }
}
