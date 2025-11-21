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
    saturn_shader,
    vertex_displacement_saturn,
    spaceship_shader,
    common::perlin_noise_3d,
    kleos_shader, // Shader de Kleos (Tierra)
};
use controls::ShipControls; // 👈 use ShipControls instead of Ship

// Para generar ángulos aleatorios de las estrellas del efecto warp
struct StarLine {
    angle: f32,
    speed: f32,
}

// Estrella en el skybox 3D
struct SkyboxStar {
    position: Vector3,  // Posición en esfera lejana
    brightness: u8,     // Brillo de la estrella
    size: f32,          // Tamaño (1.0 = pixel, 2.0 = cruz pequeña)
}

// Cometa que cruza el cielo
struct Comet {
    position: Vector3,      // Posición actual en el espacio
    velocity: Vector3,      // Velocidad y dirección
    life_time: f32,         // Tiempo total de vida
    current_time: f32,      // Tiempo transcurrido
    brightness: u8,         // Brillo del núcleo
    tail_length: f32,       // Longitud de la cola
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
    has_rings: bool,
    has_moon: bool,
}

struct TriangleDepth {
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    color: Color,
    depth: f32,
}

fn main() {
    // Obtener dimensiones del monitor actual antes de crear la ventana
    let (monitor_width, monitor_height) = unsafe {
        raylib::ffi::InitWindow(1, 1, std::ptr::null());
        let current_monitor = raylib::ffi::GetCurrentMonitor();
        let width = raylib::ffi::GetMonitorWidth(current_monitor);
        let height = raylib::ffi::GetMonitorHeight(current_monitor);
        raylib::ffi::CloseWindow();
        (width, height)
    };
    
    println!("📺 Monitor detectado: {}x{}", monitor_width, monitor_height);
    
    // Configurar ventana maximizada
    unsafe {
        raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_WINDOW_MAXIMIZED as u32);
    }
    
    let (mut rl, thread) = raylib::init()
        .size(monitor_width, monitor_height)
        .title("Solar System - Software Renderer")
        .build();

    rl.set_target_fps(60);

    // ---------------------------
    // Cargar esfera con más detalle
    // ---------------------------
    let mut sphere_tris: Vec<Vector3> = Vec::new();
    let sphere_paths = [
        "sphere_2000.obj",  // Nueva esfera con 2048 triángulos
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
    
    // Aumentar detalle: NO limitar triángulos, usar TODOS los del sphere.obj
    println!("✓ Using all {} triangles from sphere", sphere_tris.len()/3);

    // ---------------------------
    // Cargar modelo de la luna
    // ---------------------------
    let mut moon_tris: Vec<Vector3> = Vec::new();
    let moon_paths = [
        "sphere.obj",
        "../sphere.obj",
        "assets/sphere.obj",
    ];
    let mut moon_loaded = false;
    for p in moon_paths {
        if let Ok(m) = Obj::load(p) {
            moon_tris = m.get_vertex_array();
            println!("✓ Loaded moon from {} (tris={})", p, moon_tris.len()/3);
            moon_loaded = true;
            break;
        }
    }
    if !moon_loaded {
        eprintln!("⚠ No moon model found");
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
    // Planetas (con órbitas más separadas)
    // ---------------------------
    let bodies = vec![
        Body { name: "Sun", orbit_radius: 0.0, orbit_speed: 0.0, size: 18.0, color: Vector3::new(1.0,0.9,0.6), spin_speed: 0.01, has_rings: false, has_moon: false },
        Body { name: "Aeon", orbit_radius: 100.0, orbit_speed: 0.225, size: 4.5, color: Vector3::new(0.05,0.1,0.35), spin_speed: 0.15, has_rings: true, has_moon: false }, // Azul marino oscuro - ahora en posición 4
        Body { name: "Thalassa", orbit_radius: 55.0, orbit_speed: 0.4, size: 5.4, color: Vector3::new(0.9,0.7,0.4), spin_speed: 0.05, has_rings: false, has_moon: false },
        Body { name: "Kleos", orbit_radius: 75.0, orbit_speed: 0.3, size: 6.0, color: Vector3::new(0.2,0.6,0.2), spin_speed: 0.3, has_rings: false, has_moon: true }, // Verde y azul tierra
        Body { name: "Kefi", orbit_radius: 35.0, orbit_speed: 0.6, size: 3.6, color: Vector3::new(0.9,0.4,0.25), spin_speed: 0.25, has_rings: false, has_moon: false }, // Ahora en posición 1
        Body { name: "Agape", orbit_radius: 130.0, orbit_speed: 0.18, size: 11.0, color: Vector3::new(0.9,0.8,0.6), spin_speed: 0.04, has_rings: true, has_moon: false },
        Body { name: "Goliath", orbit_radius: 165.0, orbit_speed: 0.125, size: 13.2, color: Vector3::new(0.5,0.2,0.9), spin_speed: 0.025, has_rings: false, has_moon: false },
    ];

    // ---------------------------
    // Nave + cámara
    // ---------------------------
    let mut ship = ShipControls::new(Vector3 { x: 0.0, y: 0.0, z: -25.0 });
    let ship_radius = 0.35f32;

    // view mode: 1 = top, 2 = side, 3 = third-person
    let mut view_mode: i32 = 3;
    let mut free_camera_mode: bool = false;
    
    // Free camera (vista fija del sistema) - ELEVADA para mejor vista
    let mut free_cam_height: f32 = 450.0;
    let mut free_cam_distance: f32 = 300.0;
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
    
    // Generar estrellas del skybox 3D (esfera lejana)
    let mut skybox_stars: Vec<SkyboxStar> = Vec::new();
    let skybox_radius = 500.0; // Radio de la esfera del skybox
    
    // Usar semilla pseudo-aleatoria basada en índice para distribución uniforme
    for i in 0..800 {
        // Distribución esférica uniforme usando Fibonacci sphere
        let phi = std::f32::consts::PI * (3.0 - (5.0_f32).sqrt()) * i as f32;
        let y = 1.0 - (i as f32 / 399.5) * 2.0; // -1 a 1
        let radius = (1.0 - y * y).sqrt();
        
        let x = phi.cos() * radius;
        let z = phi.sin() * radius;
        
        // Normalizar y escalar al radio del skybox
        let position = Vector3::new(x, y, z) * skybox_radius;
        
        // Variación de brillo
        let brightness_seed = (i * 137 + i * i * 29) % 100;
        let brightness = if brightness_seed > 90 {
            255 // Estrellas muy brillantes (10%)
        } else if brightness_seed > 70 {
            200 // Estrellas brillantes (20%)
        } else if brightness_seed > 40 {
            150 // Estrellas normales (30%)
        } else {
            100 // Estrellas tenues (40%)
        };
        
        // Variación de tamaño
        let size = if brightness >= 200 {
            2.0 // Estrellas brillantes son más grandes
        } else {
            1.0 // Estrellas normales
        };
        
        skybox_stars.push(SkyboxStar {
            position,
            brightness,
            size,
        });
    }
    
    // Generar cometas que cruzan el cielo
    let mut comets: Vec<Comet> = Vec::new();
    let mut next_comet_spawn: f32 = 2.0; // Tiempo hasta el próximo cometa

    println!("Controls: W/S/A/D move, Space/Ctrl up/down, Arrows rotate, X nitro");
    println!("V: FREE CAMERA, T: Pause");

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        
        // Obtener dimensiones de pantalla (se actualizan si cambia el tamaño de ventana)
        let width = rl.get_screen_width();
        let height = rl.get_screen_height();

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
                free_cam_distance = 150.0;
                free_cam_height = 250.0;
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
                warp_progress = 0.001; // Iniciar animación inmediatamente
                println!("⚡ Iniciando warp a Aeon");
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
                warp_progress = 0.001; // Iniciar animación inmediatamente
                println!("⚡ Iniciando warp a Thalassa");
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
                warp_progress = 0.001; // Iniciar animación inmediatamente
                println!("⚡ Iniciando warp a Kleos");
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
                warp_progress = 0.001; // Iniciar animación inmediatamente
                println!("⚡ Iniciando warp a Kefi");
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
                warp_progress = 0.001; // Iniciar animación inmediatamente
                println!("⚡ Iniciando warp a Agape");
            }
            else if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SEVEN) {
                let angle = orbit_time * bodies[6].orbit_speed;
                warp_start_pos = ship.position;
                warp_target_pos = Vector3::new(
                    angle.cos() * bodies[6].orbit_radius,
                    0.0,
                    angle.sin() * bodies[6].orbit_radius - bodies[6].size - 15.0
                );
                warp_active = true;
                warp_progress = 0.001; // Iniciar animación inmediatamente
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
        // Actualizar cometas
        // -----------------------
        // Spawn nuevo cometa si es tiempo
        next_comet_spawn -= dt;
        if next_comet_spawn <= 0.0 {
            // Generar posición aleatoria en el borde del skybox
            let angle_h = (time * 137.5 + comets.len() as f32 * 73.2) % (2.0 * std::f32::consts::PI);
            let angle_v = ((time * 91.3 + comets.len() as f32 * 52.7) % 1.0) * std::f32::consts::PI - std::f32::consts::PI / 2.0;
            
            let skybox_spawn_radius = 480.0;
            let start_x = angle_h.cos() * angle_v.cos() * skybox_spawn_radius;
            let start_y = angle_v.sin() * skybox_spawn_radius;
            let start_z = angle_h.sin() * angle_v.cos() * skybox_spawn_radius;
            
            // Dirección hacia el lado opuesto (cruzando el cielo)
            let dir_angle_h = angle_h + std::f32::consts::PI + (((time * 43.7) % 1.0) - 0.5) * 0.5;
            let dir_angle_v = -angle_v + (((time * 67.3) % 1.0) - 0.5) * 0.3;
            
            let velocity = Vector3::new(
                dir_angle_h.cos() * dir_angle_v.cos(),
                dir_angle_v.sin(),
                dir_angle_h.sin() * dir_angle_v.cos()
            ) * (120.0 + ((time * 29.1) % 1.0) * 60.0); // Velocidad alta constante: 120-180
            
            let brightness_seed = ((time * 113.7) % 1.0);
            let brightness = if brightness_seed > 0.7 {
                220
            } else {
                180
            };
            
            comets.push(Comet {
                position: Vector3::new(start_x, start_y, start_z),
                velocity,
                life_time: 8.0 + ((time * 47.3) % 1.0) * 4.0, // Duración variable: 8-12 segundos
                current_time: 0.0,
                brightness,
                tail_length: 100.0 + ((time * 83.5) % 1.0) * 80.0, // Colas muy largas: 100-180
            });
            
            // Próximo spawn entre 1 y 4 segundos (más frecuente y aleatorio)
            next_comet_spawn = 1.0 + ((time * 127.9) % 1.0) * 3.0;
        }
        
        // Actualizar posición de cometas existentes
        for comet in comets.iter_mut() {
            comet.current_time += dt;
            
            // Calcular factor de desaceleración (empieza en 1.0 y termina en 0.3)
            let life_progress = comet.current_time / comet.life_time;
            let speed_factor = 1.0 - (life_progress * 0.7); // De 1.0 a 0.3
            
            comet.position = Vector3::new(
                comet.position.x + comet.velocity.x * dt * speed_factor,
                comet.position.y + comet.velocity.y * dt * speed_factor,
                comet.position.z + comet.velocity.z * dt * speed_factor,
            );
        }
        
        // Eliminar cometas que terminaron su vida
        comets.retain(|c| c.current_time < c.life_time);

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
        
        // Nebula/Galaxy background animation
        let nebula_time = time * 0.05; // Slow animation
        let pixel_step = 4; // Render every 4th pixel for performance
        
        for py in (0..height).step_by(pixel_step) {
            for px in (0..width).step_by(pixel_step) {
                // Normalized coordinates
                let nx = (px as f32 / width as f32) * 2.0 - 1.0;
                let ny = (py as f32 / height as f32) * 2.0 - 1.0;
                
                // Multiple octaves of noise for cloud-like patterns
                let scale1 = 1.5;
                let scale2 = 3.0;
                let scale3 = 6.0;
                
                let noise1 = perlin_noise_3d((nx * scale1 + nebula_time) as f64, (ny * scale1) as f64, (nebula_time * 0.5) as f64);
                let noise2 = perlin_noise_3d((nx * scale2 - nebula_time * 0.7) as f64, (ny * scale2) as f64, (nebula_time * 0.3) as f64);
                let noise3 = perlin_noise_3d((nx * scale3) as f64, (ny * scale3 + nebula_time) as f64, (nebula_time * 0.2) as f64);
                
                // Combine noise layers
                let combined = (noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2).clamp(-1.0, 1.0);
                
                // Map to red and blue regions
                let red_intensity = if combined > 0.2 {
                    ((combined - 0.2) * 80.0).min(60.0) as u8
                } else {
                    0
                };
                
                let blue_intensity = if combined < -0.2 {
                    ((combined.abs() - 0.2) * 100.0).min(80.0) as u8
                } else {
                    0
                };
                
                // Add some purple/magenta where red and blue overlap
                let purple_factor = if combined.abs() < 0.15 {
                    20
                } else {
                    0
                };
                
                if red_intensity > 0 || blue_intensity > 0 || purple_factor > 0 {
                    let nebula_color = Color::new(
                        red_intensity + purple_factor,
                        purple_factor / 2,
                        blue_intensity + purple_factor,
                        180
                    );
                    
                    // Draw block of pixels for the step size
                    d.draw_rectangle(px as i32, py as i32, pixel_step as i32, pixel_step as i32, nebula_color);
                }
            }
        }
        
        d.draw_text("Solar System - Software Renderer", 20, 20, 24, Color::LIGHTGRAY);

        let mode_indicator = if free_camera_mode { 
            "FREE CAMERA - Elevated View" 
        } else { 
            "EXPLORATION" 
        };
        let status = if paused { "- PAUSED -" } else { "- Active -" };

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
            d.draw_rectangle(width - 260, 10, 250, 200, Color::new(0, 0, 0, 180));
            d.draw_rectangle_lines(width - 260, 10, 250, 200, Color::new(100, 200, 255, 255));
            d.draw_text("INSTANT WARP", width - 250, 20, 20, Color::new(100, 200, 255, 255));
            d.draw_text("1 - Sun", width - 240, 50, 18, Color::new(255, 220, 100, 255));
            d.draw_text("2 - Aeon", width - 240, 72, 18, Color::new(180, 180, 190, 255));
            d.draw_text("3 - Thalassa", width - 240, 94, 18, Color::new(230, 180, 100, 255));
            d.draw_text("4 - Kleos", width - 240, 116, 18, Color::new(80, 150, 230, 255));
            d.draw_text("5 - Kefi", width - 240, 138, 18, Color::new(230, 100, 60, 255));
            d.draw_text("6 - Agape", width - 240, 160, 18, Color::new(230, 200, 150, 255));
            d.draw_text("7 - Goliath", width - 240, 182, 18, Color::new(150, 80, 200, 255));
        }

        // Skybox de estrellas 3D
        for star in &skybox_stars {
            // Posición relativa a la cámara
            let mut star_cam = Vector3::new(
                star.position.x - cam_world_x,
                star.position.y - cam_y,
                star.position.z - cam_world_z
            );
            
            // Aplicar rotación de cámara
            star_cam = rotate_x(star_cam, -cam_pitch);
            star_cam = rotate_y(star_cam, -cam_yaw);
            
            // Proyectar a pantalla
            if let Some((sx, sy)) = perspective_project(star_cam, width as f32, height as f32, 70.0) {
                // Verificar que esté en pantalla
                if sx >= 0.0 && sx < width as f32 && sy >= 0.0 && sy < height as f32 {
                    let star_color = Color::new(200, 200, 255, star.brightness);
                    
                    if star.size >= 2.0 {
                        // Estrellas brillantes: dibujar cruz pequeña
                        let sx_i = sx as i32;
                        let sy_i = sy as i32;
                        d.draw_pixel(sx_i, sy_i, star_color);
                        d.draw_pixel(sx_i + 1, sy_i, star_color);
                        d.draw_pixel(sx_i - 1, sy_i, star_color);
                        d.draw_pixel(sx_i, sy_i + 1, star_color);
                        d.draw_pixel(sx_i, sy_i - 1, star_color);
                    } else {
                        // Estrellas normales: pixel simple
                        d.draw_pixel(sx as i32, sy as i32, star_color);
                    }
                }
            }
        }
        
        // Renderizar cometas
        for comet in &comets {
            // Fade in/out basado en el tiempo de vida
            let life_progress = comet.current_time / comet.life_time;
            
            // Fade in rápido al inicio (primeros 0.2 segundos)
            // Fade out gradual al final (últimos 0.8 segundos)
            let alpha = if life_progress < 0.067 { // Primeros 0.2 segundos
                (life_progress / 0.067 * 255.0) as u8
            } else if life_progress > 0.733 { // Últimos 0.8 segundos
                ((1.0 - life_progress) / 0.267 * 255.0) as u8
            } else {
                255
            };
            
            if alpha == 0 { continue; }
            
            // Posición del núcleo relativa a la cámara
            let mut nucleus_cam = Vector3::new(
                comet.position.x - cam_world_x,
                comet.position.y - cam_y,
                comet.position.z - cam_world_z
            );
            
            // Aplicar rotación de cámara
            nucleus_cam = rotate_x(nucleus_cam, -cam_pitch);
            nucleus_cam = rotate_y(nucleus_cam, -cam_yaw);
            
            // Proyectar núcleo
            if let Some((nx, ny)) = perspective_project(nucleus_cam, width as f32, height as f32, 70.0) {
                if nx >= 0.0 && nx < width as f32 && ny >= 0.0 && ny < height as f32 {
                    let brightness_adjusted = ((comet.brightness as f32 * alpha as f32) / 255.0) as u8;
                    
                    // Dibujar núcleo brillante
                    let nucleus_color = Color::new(255, 240, 200, brightness_adjusted);
                    let nx_i = nx as i32;
                    let ny_i = ny as i32;
                    
                    // Núcleo más grande
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            d.draw_pixel(nx_i + dx, ny_i + dy, nucleus_color);
                        }
                    }
                    
                    // Dibujar cola (líneas conectadas para movimiento fluido)
                    let tail_segments = 25;
                    let mut prev_point: Option<(i32, i32)> = Some((nx_i, ny_i));
                    
                    for i in 1..=tail_segments {
                        let t = i as f32 / tail_segments as f32;
                        
                        // Posición en la cola
                        let tail_pos = Vector3::new(
                            comet.position.x - comet.velocity.x * t * comet.tail_length / comet.velocity.length(),
                            comet.position.y - comet.velocity.y * t * comet.tail_length / comet.velocity.length(),
                            comet.position.z - comet.velocity.z * t * comet.tail_length / comet.velocity.length(),
                        );
                        
                        let mut tail_cam = Vector3::new(
                            tail_pos.x - cam_world_x,
                            tail_pos.y - cam_y,
                            tail_pos.z - cam_world_z
                        );
                        
                        tail_cam = rotate_x(tail_cam, -cam_pitch);
                        tail_cam = rotate_y(tail_cam, -cam_yaw);
                        
                        if let Some((tx, ty)) = perspective_project(tail_cam, width as f32, height as f32, 70.0) {
                            if tx >= 0.0 && tx < width as f32 && ty >= 0.0 && ty < height as f32 {
                                let tx_i = tx as i32;
                                let ty_i = ty as i32;
                                
                                // La cola se desvanece gradualmente
                                let tail_alpha = ((1.0 - t) * alpha as f32 * 0.8) as u8;
                                
                                // Color de la cola: azul-blanco con gradiente
                                let tail_color = Color::new(
                                    (200.0 + t * 55.0) as u8,
                                    (220.0 + t * 35.0) as u8,
                                    255,
                                    tail_alpha
                                );
                                
                                // Conectar con el punto anterior usando línea
                                if let Some((px, py)) = prev_point {
                                    d.draw_line_ex(
                                        Vector2::new(px as f32, py as f32),
                                        Vector2::new(tx_i as f32, ty_i as f32),
                                        if t < 0.4 { 2.0 } else { 1.0 },
                                        tail_color
                                    );
                                }
                                
                                prev_point = Some((tx_i, ty_i));
                            } else {
                                prev_point = None;
                            }
                        } else {
                            prev_point = None;
                        }
                    }
                }
            }
        }

        // Órbitas
        for (idx, b) in bodies.iter().enumerate() {
            if idx == 0 { continue; }
            let orbit_points = 200; // Aumentado para órbitas más suaves y redondas
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
                        let orbit_color = if idx == 1 {
                            // Órbita AZUL para Aeon
                            Color::new(50, 150, 255, 200)
                        } else {
                            Color::new(
                                (b.color.x * 150.0) as u8,
                                (b.color.y * 150.0) as u8,
                                (b.color.z * 150.0) as u8,
                                180
                            )
                        };
                        d.draw_line_v(Vector2::new(x1, y1), Vector2::new(x2, y2), orbit_color);
                    }
                }
            }
        }
        
        // Anillos de planetas - renderizar como líneas antes de los planetas
        for (idx, b) in bodies.iter().enumerate() {
            if !b.has_rings { continue; }
            
            let angle = orbit_time * b.orbit_speed;
            let bx = angle.cos() * b.orbit_radius;
            let bz = angle.sin() * b.orbit_radius;
            
            // Aeon (idx 1): anillos giratorios en múltiples orientaciones
            if idx == 1 {
                // Anillos giratorios que crean efecto de esfera - ahora con grosor
                let num_ring_orbits = 6; // 6 anillos en diferentes ángulos
                
                for orbit_idx in 0..num_ring_orbits {
                    // Ángulo de inclinación de cada anillo orbital
                    let orbit_tilt = (orbit_idx as f32 / num_ring_orbits as f32) * std::f32::consts::PI;
                    // Velocidad de rotación de cada anillo
                    let rotation_speed = 2.0 + orbit_idx as f32 * 0.5;
                    let rotation_angle = time * rotation_speed;
                    
                    // Crear 2 anillos concéntricos para dar grosor
                    for ring_layer in 0..2 {
                        let radius = b.size * 1.5 + (ring_layer as f32 * 0.15); // Segundo anillo ligeramente más grande
                        let segments = 80;
                        
                        // Color NEÓN azul brillante para Aeon
                        let brightness = 255 - orbit_idx * 25;
                        let pulse = ((time * 3.0).sin() * 0.2 + 0.8); // Pulsación
                        let ring_color = Color::new(
                            (30.0 * pulse) as u8,
                            (150.0 * pulse) as u8,
                            (brightness as f32 * pulse) as u8,
                            (220 - orbit_idx * 20).max(150) as u8
                        );
                        
                        for i in 0..segments {
                            let angle1 = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
                            let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 2.0;
                            
                            // Crear puntos del anillo en espacio local
                            let mut p1 = Vector3::new(
                                angle1.cos() * radius,
                                0.0,
                                angle1.sin() * radius
                            );
                            
                            let mut p2 = Vector3::new(
                                angle2.cos() * radius,
                                0.0,
                                angle2.sin() * radius
                            );
                            
                            // Aplicar inclinación del anillo
                            p1 = rotate_x(p1, orbit_tilt);
                            p2 = rotate_x(p2, orbit_tilt);
                            
                            // Aplicar rotación animada
                            p1 = rotate_y(p1, rotation_angle);
                            p2 = rotate_y(p2, rotation_angle);
                            
                            // Trasladar a la posición del planeta
                            p1.x += bx;
                            p1.z += bz;
                            p2.x += bx;
                            p2.z += bz;
                            
                            // Transformar a espacio de cámara
                            let mut c1 = Vector3::new(p1.x - cam_world_x, p1.y - cam_y, p1.z - cam_world_z);
                            let mut c2 = Vector3::new(p2.x - cam_world_x, p2.y - cam_y, p2.z - cam_world_z);
                            
                            c1 = rotate_x(c1, -cam_pitch);
                            c1 = rotate_y(c1, -cam_yaw);
                            c2 = rotate_x(c2, -cam_pitch);
                            c2 = rotate_y(c2, -cam_yaw);
                            
                            // Proyectar y dibujar línea
                            if let Some((x1, y1)) = perspective_project(c1, width as f32, height as f32, 70.0) {
                                if let Some((x2, y2)) = perspective_project(c2, width as f32, height as f32, 70.0) {
                                    d.draw_line_v(Vector2::new(x1, y1), Vector2::new(x2, y2), ring_color);
                                }
                            }
                        }
                    }
                }
            }
            // Agape (idx 5): anillos estáticos tradicionales
            else if idx == 5 {
                // Crear muchas líneas concéntricas para simular los anillos
                let ring_start = b.size * 1.3;
                let ring_end = b.size * 2.4;
                let num_ring_lines = 60; // Muchas líneas para efecto denso
                
                for ring_idx in 0..num_ring_lines {
                    let t = ring_idx as f32 / num_ring_lines as f32;
                    let radius = ring_start + (ring_end - ring_start) * t;
                    
                    // Color del anillo con variación
                    let brightness = if ring_idx < 15 {
                        240 - ring_idx * 4
                    } else if ring_idx < 35 {
                        180 - (ring_idx - 15) * 2
                    } else {
                        150 - (ring_idx - 35) * 3
                    };
                    
                    let ring_color = Color::new(
                        brightness.max(100) as u8,
                        (brightness.max(100) as f32 * 0.85) as u8,
                        (brightness.max(100) as f32 * 0.65) as u8,
                        255
                    );
                    
                    // Dibujar círculo completo del anillo
                    let segments = 120;
                    for i in 0..segments {
                        let angle1 = (i as f32 / segments as f32) * std::f32::consts::PI * 2.0;
                        let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::PI * 2.0;
                        
                        let p1 = Vector3::new(
                            angle1.cos() * radius + bx,
                            0.0,
                            angle1.sin() * radius + bz
                        );
                        
                        let p2 = Vector3::new(
                            angle2.cos() * radius + bx,
                            0.0,
                            angle2.sin() * radius + bz
                        );
                        
                        // Transformar a espacio de cámara
                        let mut c1 = Vector3::new(p1.x - cam_world_x, p1.y - cam_y, p1.z - cam_world_z);
                        let mut c2 = Vector3::new(p2.x - cam_world_x, p2.y - cam_y, p2.z - cam_world_z);
                        
                        c1 = rotate_x(c1, -cam_pitch);
                        c1 = rotate_y(c1, -cam_yaw);
                        c2 = rotate_x(c2, -cam_pitch);
                        c2 = rotate_y(c2, -cam_yaw);
                        
                        // Proyectar y dibujar línea
                        if let Some((x1, y1)) = perspective_project(c1, width as f32, height as f32, 70.0) {
                            if let Some((x2, y2)) = perspective_project(c2, width as f32, height as f32, 70.0) {
                                d.draw_line_v(Vector2::new(x1, y1), Vector2::new(x2, y2), ring_color);
                            }
                        }
                    }
                }
            }
        }

        // Triángulos de planetas
        let mut triangles: Vec<TriangleDepth> = Vec::new();

        // Luego agregar triángulos de planetas

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
                        // Aeon
                        v0 = vertex_displacement_mercury(v0, time);
                        v1 = vertex_displacement_mercury(v1, time);
                        v2 = vertex_displacement_mercury(v2, time);
                    },
                    2 => {
                        // Thalassa
                        v0 = vertex_displacement_venus(v0, time);
                        v1 = vertex_displacement_venus(v1, time);
                        v2 = vertex_displacement_venus(v2, time);
                    },
                    3 => {
                        // Kleos
                        v0 = vertex_displacement_earth(v0, time);
                        v1 = vertex_displacement_earth(v1, time);
                        v2 = vertex_displacement_earth(v2, time);
                    },
                    4 => {
                        // Kefi
                        v0 = vertex_displacement_mars(v0, time);
                        v1 = vertex_displacement_mars(v1, time);
                        v2 = vertex_displacement_mars(v2, time);
                    },
                    5 => {
                        // Agape
                        v0 = vertex_displacement_saturn(v0, time);
                        v1 = vertex_displacement_saturn(v1, time);
                        v2 = vertex_displacement_saturn(v2, time);
                    },
                    6 => {
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
                    3 => kleos_shader(&fragment, time, b.color), // Kleos - Tierra
                    4 => perlin_planet_shader(&fragment, time, b.color),
                    5 => {
                        let uniforms = Uniforms::new(time, 1.0);
                        saturn_shader(&fragment, &uniforms)
                    },
                    6 => planet_shader(&fragment, time, b.color), 
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

        // -----------------------
        // Renderizar la luna de Kleos
        // -----------------------
        if moon_loaded {
            let kleos_idx = 3;
            let b = &bodies[kleos_idx];
            
            // Posición de Kleos en su órbita
            let angle = orbit_time * b.orbit_speed;
            let kleos_x = angle.cos() * b.orbit_radius;
            let kleos_z = angle.sin() * b.orbit_radius;
            
            // Órbita de la luna alrededor de Kleos
            let moon_orbit_radius = b.size * 2.5; // Distancia de la luna a Kleos
            let moon_orbit_speed = 2.0; // Velocidad orbital de la luna
            let moon_angle = orbit_time * moon_orbit_speed;
            
            // Posición de la luna en el mundo con inclinación de 45°
            let moon_orbit_tilt = std::f32::consts::PI / 4.0; // 45 grados
            let mut moon_offset = Vector3::new(
                moon_angle.cos() * moon_orbit_radius,
                0.0,
                moon_angle.sin() * moon_orbit_radius
            );
            // Aplicar inclinación
            moon_offset = rotate_x(moon_offset, moon_orbit_tilt);
            
            let moon_x = kleos_x + moon_offset.x;
            let moon_y = moon_offset.y;
            let moon_z = kleos_z + moon_offset.z;
            let moon_size = b.size * 0.27; // Tamaño de la luna (27% del planeta)
            
            // Renderizar la luna
            for tri_idx in (0..moon_tris.len()).step_by(3) {
                let mut v0 = moon_tris[tri_idx] * moon_size;
                let mut v1 = moon_tris[tri_idx+1] * moon_size;
                let mut v2 = moon_tris[tri_idx+2] * moon_size;
                
                // Rotación de la luna
                let moon_spin_angle = time * 0.1;
                v0 = rotate_y(v0, moon_spin_angle);
                v1 = rotate_y(v1, moon_spin_angle);
                v2 = rotate_y(v2, moon_spin_angle);
                
                // Trasladar a la posición de la luna
                v0.x += moon_x; v0.y += moon_y; v0.z += moon_z;
                v1.x += moon_x; v1.y += moon_y; v1.z += moon_z;
                v2.x += moon_x; v2.y += moon_y; v2.z += moon_z;
                
                // Transformar a espacio de cámara
                let mut c0 = Vector3::new(v0.x - cam_world_x, v0.y - cam_y, v0.z - cam_world_z);
                let mut c1 = Vector3::new(v1.x - cam_world_x, v1.y - cam_y, v1.z - cam_world_z);
                let mut c2 = Vector3::new(v2.x - cam_world_x, v2.y - cam_y, v2.z - cam_world_z);
                
                c0 = rotate_x(c0, -cam_pitch);
                c1 = rotate_x(c1, -cam_pitch);
                c2 = rotate_x(c2, -cam_pitch);
                c0 = rotate_y(c0, -cam_yaw);
                c1 = rotate_y(c1, -cam_yaw);
                c2 = rotate_y(c2, -cam_yaw);
                
                // Backface culling
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
                if dot >= 0.0 { continue; }
                
                // Proyección
                let p0 = perspective_project(c0, width as f32, height as f32, 70.0);
                let p1 = perspective_project(c1, width as f32, height as f32, 70.0);
                let p2 = perspective_project(c2, width as f32, height as f32, 70.0);
                if p0.is_none() || p1.is_none() || p2.is_none() { continue; }
                let (x0,y0) = p0.unwrap(); let (x1,y1) = p1.unwrap(); let (x2,y2) = p2.unwrap();
                
                // Shader de la luna (color gris rocoso)
                let len = (normal.x*normal.x + normal.y*normal.y + normal.z*normal.z).sqrt();
                let n = if len > 0.0001 {
                    normal / len
                } else {
                    Vector3::new(0.0,1.0,0.0)
                };
                
                let light_dir = Vector3::new(0.5, 1.0, -0.3);
                let light_len = (light_dir.x*light_dir.x + light_dir.y*light_dir.y + light_dir.z*light_dir.z).sqrt();
                let light_normalized = light_dir / light_len;
                let diff = (n.x * light_normalized.x + n.y * light_normalized.y + n.z * light_normalized.z).max(0.2);
                
                // Color gris luna con textura rocosa
                let world_center = (v0 + v1 + v2) / 3.0;
                let noise_val = ((world_center.x * 15.0).sin() * (world_center.z * 15.0).cos() + 1.0) * 0.5;
                let moon_base = 0.6 + noise_val * 0.15;
                
                let fill = Color::new(
                    (moon_base * diff * 255.0) as u8,
                    (moon_base * diff * 255.0) as u8,
                    (moon_base * diff * 240.0) as u8, // Ligeramente menos azul
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

        // -----------------------
        // Calcular posición y profundidad de Aeon antes de dibujar
        // -----------------------
        let aeon_idx = 1;
        let angle = orbit_time * bodies[aeon_idx].orbit_speed;
        let aeon_x = angle.cos() * bodies[aeon_idx].orbit_radius;
        let aeon_z = angle.sin() * bodies[aeon_idx].orbit_radius;
        
        // Posición relativa a la cámara
        let mut aeon_cam = Vector3::new(
            aeon_x - cam_world_x,
            0.0 - cam_y,
            aeon_z - cam_world_z
        );
        
        aeon_cam = rotate_x(aeon_cam, -cam_pitch);
        aeon_cam = rotate_y(aeon_cam, -cam_yaw);
        
        let aeon_depth = aeon_cam.z; // Profundidad de Aeon en espacio de cámara
        
        // -----------------------
        // Dibujar triángulos intercalando el glow de Aeon
        // -----------------------
        let mut aeon_glow_drawn = false;
        
        for tri in &triangles {
            // Si llegamos a triángulos más cercanos que Aeon, dibujar el glow primero
            if !aeon_glow_drawn && tri.depth < aeon_depth {
                // Proyectar posición de Aeon y dibujar glow (reducido)
                if let Some((aeon_screen_x, aeon_screen_y)) = perspective_project(aeon_cam, width as f32, height as f32, 70.0) {
                    let glow_layers = 6; // Menos capas
                    for layer in 1..=glow_layers {
                        let radius = bodies[aeon_idx].size * (1.3 + layer as f32 * 0.15); // Más pequeño
                        let alpha = (180 - layer * 25).max(10) as u8; // Menos intenso
                        
                        let distance_to_cam = (aeon_cam.x * aeon_cam.x + aeon_cam.y * aeon_cam.y + aeon_cam.z * aeon_cam.z).sqrt();
                        let screen_radius = if distance_to_cam > 0.1 {
                            (radius * 500.0 / distance_to_cam).max(3.0)
                        } else {
                            10.0
                        };
                        
                        let glow_color = Color::new(20, 80, 200, alpha / ((layer as u8) + 1)); // Menos brillante
                        d.draw_circle(aeon_screen_x as i32, aeon_screen_y as i32, screen_radius, glow_color);
                    }
                }
                aeon_glow_drawn = true;
            }
            
            // Dibujar triángulo
            d.draw_triangle(
                Vector2::new(tri.p0.0, tri.p0.1),
                Vector2::new(tri.p1.0, tri.p1.1),
                Vector2::new(tri.p2.0, tri.p2.1),
                tri.color
            );
        }
        
        // Si no se dibujó el glow (Aeon está más atrás que todos), dibujarlo al final
        if !aeon_glow_drawn {
            if let Some((aeon_screen_x, aeon_screen_y)) = perspective_project(aeon_cam, width as f32, height as f32, 70.0) {
                let glow_layers = 6; // Menos capas
                for layer in 1..=glow_layers {
                    let radius = bodies[aeon_idx].size * (1.3 + layer as f32 * 0.15); // Más pequeño
                    let alpha = (180 - layer * 25).max(10) as u8; // Menos intenso
                    
                    let distance_to_cam = (aeon_cam.x * aeon_cam.x + aeon_cam.y * aeon_cam.y + aeon_cam.z * aeon_cam.z).sqrt();
                    let screen_radius = if distance_to_cam > 0.1 {
                        (radius * 500.0 / distance_to_cam).max(3.0)
                    } else {
                        10.0
                    };
                    
                    let glow_color = Color::new(20, 80, 200, alpha / ((layer as u8) + 1)); // Menos brillante
                    d.draw_circle(aeon_screen_x as i32, aeon_screen_y as i32, screen_radius, glow_color);
                }
            }
        }

        // -----------------------
        // Nave - visible en ambos modos
        // -----------------------
        if !ship_tris.is_empty() {
            if free_camera_mode {
                // En modo cámara libre, mostrar la nave en el mundo desde arriba
                let ship_scale = 0.5f32;
                
                for i in (0..ship_tris.len()).step_by(3) {
                    if i + 2 >= ship_tris.len() { break; }
                    let mut v0 = ship_tris[i] * ship_scale;
                    let mut v1 = ship_tris[i+1] * ship_scale;
                    let mut v2 = ship_tris[i+2] * ship_scale;

                    // Rotar la nave según su orientación
                    let ship_rotation = ship.yaw;
                    v0 = rotate_y(v0, ship_rotation);
                    v1 = rotate_y(v1, ship_rotation);
                    v2 = rotate_y(v2, ship_rotation);
                    
                    // Posicionar en el mundo
                    v0.x += ship.position.x;
                    v0.y += ship.position.y;
                    v0.z += ship.position.z;
                    v1.x += ship.position.x;
                    v1.y += ship.position.y;
                    v1.z += ship.position.z;
                    v2.x += ship.position.x;
                    v2.y += ship.position.y;
                    v2.z += ship.position.z;
                    
                    // Transformar a espacio de cámara
                    let mut c0 = Vector3::new(v0.x - cam_world_x, v0.y - cam_y, v0.z - cam_world_z);
                    let mut c1 = Vector3::new(v1.x - cam_world_x, v1.y - cam_y, v1.z - cam_world_z);
                    let mut c2 = Vector3::new(v2.x - cam_world_x, v2.y - cam_y, v2.z - cam_world_z);
                    
                    c0 = rotate_x(c0, -cam_pitch);
                    c0 = rotate_y(c0, -cam_yaw);
                    c1 = rotate_x(c1, -cam_pitch);
                    c1 = rotate_y(c1, -cam_yaw);
                    c2 = rotate_x(c2, -cam_pitch);
                    c2 = rotate_y(c2, -cam_yaw);
                    
                    // Backface culling
                    let e1 = c1 - c0;
                    let e2 = c2 - c0;
                    let n = Vector3::new(
                        e1.y * e2.z - e1.z * e2.y,
                        e1.z * e2.x - e1.x * e2.z,
                        e1.x * e2.y - e1.y * e2.x,
                    );
                    let center = (c0 + c1 + c2) / 3.0;
                    let view_dir = -center;
                    let dotp = n.x * view_dir.x + n.y * view_dir.y + n.z * view_dir.z;
                    if dotp >= 0.0 { continue; }
                    
                    // Proyectar
                    if let Some((x0, y0)) = perspective_project(c0, width as f32, height as f32, 70.0) {
                        if let Some((x1, y1)) = perspective_project(c1, width as f32, height as f32, 70.0) {
                            if let Some((x2, y2)) = perspective_project(c2, width as f32, height as f32, 70.0) {
                                // Color de la nave
                                let ship_color = Color::new(180, 180, 200, 255);
                                d.draw_triangle(
                                    Vector2::new(x0, y0),
                                    Vector2::new(x1, y1),
                                    Vector2::new(x2, y2),
                                    ship_color
                                );
                            }
                        }
                    }
                }
            } else {
                // En modo exploración, mostrar cockpit
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
        }
    }
}
