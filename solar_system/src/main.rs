use raylib::prelude::*;
mod obj;
mod shader;
mod controls;

use obj::Obj;
use raylib::math::Vector3;
use shader::{Fragment, Uniforms, star_shader, planet_shader, spaceship_shader, 
             cellular_planet_shader, simplex_planet_shader, voronoi_planet_shader, 
             perlin_planet_shader, Vector3Ext};
use controls::ShipControls;

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

    // Load sphere geometry (triangles)
    let mut sphere_tris: Vec<Vector3> = Vec::new();
    let sphere_paths = [
        "assets/sphere.obj",
        "../planet_shaders/assets/sphere.obj",
        "../star_shader/assets/sphere.obj",
        "sphere.obj",
    ];
    let mut loaded = false;
    for p in sphere_paths {
        if let Ok(m) = Obj::load(p) {
            sphere_tris = m.get_vertex_array();
            println!("Loaded sphere from {} (tris={})", p, sphere_tris.len()/3);
            loaded = true; break;
        }
    }
    if !loaded { eprintln!("Could not load sphere.obj — place it in assets/"); return; }

    // Load spaceship model (required) - prioritize local spaceship.obj
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

    // Define system bodies in the ecliptic plane (y = 0) - CUÁDRUPLE TAMAÑO (×4 del original)
    let bodies = vec![
        Body { name: "Sun", orbit_radius: 0.0, orbit_speed: 0.0, size: 10.0, color: Vector3::new(1.0,0.9,0.6), spin_speed: 0.02 },
        Body { name: "Mercury", orbit_radius: 4.0, orbit_speed: 1.2, size: 2.4, color: Vector3::new(0.6,0.6,0.65), spin_speed: 0.3 },
        Body { name: "Venus", orbit_radius: 6.0, orbit_speed: 0.8, size: 3.6, color: Vector3::new(0.9,0.7,0.4), spin_speed: 0.1 },
        Body { name: "Earth", orbit_radius: 8.5, orbit_speed: 0.6, size: 4.0, color: Vector3::new(0.2,0.5,0.9), spin_speed: 0.6 },
        Body { name: "Mars", orbit_radius: 11.0, orbit_speed: 0.45, size: 3.0, color: Vector3::new(0.9,0.4,0.25), spin_speed: 0.5 },
        Body { name: "Goliath", orbit_radius: 14.0, orbit_speed: 0.25, size: 8.8, color: Vector3::new(0.6,0.3,0.8), spin_speed: 0.05 },
    ];

    // Ship / camera state: ship position and movement
    let mut ship_x: f32 = 0.0;
    let mut ship_z: f32 = -8.0;
    let mut ship_y: f32 = 0.0; // spaceship can now move vertically
    let mut ship_vel_x: f32 = 0.0;
    let mut ship_vel_z: f32 = 0.0;
    let mut ship_vel_y: f32 = 0.0; // vertical velocity
    let ship_radius = 0.35f32; // collision radius

    // Camera parameters (views will follow ship)
    let mut cam_y: f32 = 1.5; // height above ecliptic (for cockpit/free view)
    let mut ship_yaw: f32 = 0.0; // dirección de la nave (para movimiento WASD)
    let mut cam_yaw: f32 = 0.0; // rotación de la cámara (flechas, solo visual)
    let mut cam_pitch: f32 = 0.0; // camera pitch (used for top view)

    let thrust: f32 = 0.3; // acceleration (normal) - REDUCIDO A LA MITAD (era 0.6)
    let nitro_thrust: f32 = 0.6; // MITAD del anterior (era 1.2)
    let friction: f32 = 0.88; // velocity damping per frame

    // view mode: 1 = top, 2 = side, 3 = third-person/cockpit (default)
    let mut view_mode: i32 = 3;

    // Pause and nitro state
    let mut paused: bool = false;
    let mut nitro_active: bool = false;

    // Orbits animation
    let mut time: f32 = 0.0;

    // Warp points (centers of each body)
    let mut animated_warp: bool = true;
    let mut warp_progress: f32 = 0.0;
    let warp_duration: f32 = 0.8;
    let mut warp_from = (ship_x, ship_z);
    let mut warp_to = (ship_x, ship_z);

    // Collision is enabled by default and cannot be disabled
    let collision_enabled = true;

    // Inicializar sistema de controles
    let mut controls = ShipControls::new();

    println!("Controls: WASD move (relative to camera), SPACE up, SHIFT down, Arrows rotate camera");
    println!("V: cycle views | 1-6: warp to bodies | R: toggle warp anim | X: NITRO BOOST");
    println!("T: PAUSE/UNPAUSE | Ship position ({:.1}, {:.1}, {:.1})", ship_x, ship_y, ship_z);

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        
        // Toggle pause with T
        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_T) {
            paused = !paused;
            println!("⏸ Paused: {}", paused);
        }
        
        // Only update time and physics if not paused
        if !paused {
            time += dt;
        }

        // Actualizar controles
        controls.update(&rl);
        
        // Nitro boost basado en controles
        nitro_active = controls.nitro_active;
        let current_thrust = if nitro_active { nitro_thrust } else { thrust };
        
        // Actualizar rotación con flechas LEFT/RIGHT
        // Solo afecta cam_yaw (dirección de movimiento)
        cam_yaw += controls.rotation * 0.04;
        
        if controls.rotation != 0.0 {
            println!("🔄 cam_yaw (flechas): {:.1}°", cam_yaw.to_degrees());
        }
        
        // Aplicar velocidad usando cam_yaw (NO cam_forward)
        let (dv_x, dv_z, dv_y) = controls.apply_to_velocity(cam_yaw, current_thrust);

        ship_vel_x += dv_x;
        ship_vel_y += dv_y;
        ship_vel_z += dv_z;
        
        // apply friction
        ship_vel_x *= friction.powf(dt * 60.0);
        ship_vel_z *= friction.powf(dt * 60.0);
        ship_vel_y *= friction.powf(dt * 60.0);

        // integrate ship position
        ship_x += ship_vel_x * dt * 60.0;
        ship_y += ship_vel_y * dt * 60.0;
        ship_z += ship_vel_z * dt * 60.0;

        // Compute camera world position (anchor) based on view_mode
        let cam_world_x: f32;
        let cam_world_z: f32;
        match view_mode {
            1 => {
                // top: camera directly above the ship
                cam_world_x = ship_x;
                cam_world_z = ship_z;
                cam_y = ship_y + 18.0; // Follow ship Y position with offset
            }
            2 => {
                // side: offset on X to show the ecliptic horizontally
                cam_world_x = ship_x - 12.0;
                cam_world_z = ship_z;
                cam_y = ship_y + 4.0; // Follow ship Y position with offset
            }
            _ => {
                // third-person/cockpit: behind the ship using cam_yaw
                let behind_dist = 3.0;
                // Cámara atrás de la nave según cam_yaw
                cam_world_x = ship_x - cam_yaw.sin() * behind_dist;
                cam_world_z = ship_z - cam_yaw.cos() * behind_dist;
                cam_y = ship_y + 1.5; // Follow ship Y position with offset
            }
        }

        // Begin drawing
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(4,4,16,255));
        d.draw_text("Solar System - Software Renderer", 20, 20, 24, Color::LIGHTGRAY);
        
        let view_name = match view_mode {
            1 => "TOP-DOWN",
            2 => "SIDE",
            _ => "THIRD-PERSON",
        };
        let status = if paused { "⏸ PAUSED" } else if nitro_active { "🚀 NITRO!" } else { "▶ Active" };
        
        d.draw_text(&format!("Ship: ({:.2},{:.2})  Yaw:{:.2}  View:{} {}", ship_x, ship_z, cam_yaw, view_name, status), 20, 50, 18, Color::WHITE);
        d.draw_text("W/S/A/D: move | Arrows: rotate | V: cycle view | 1-6: warp | T: pause | X: nitro", 20, 74, 16, Color::DARKGRAY);

        // Draw simple starfield background points
        for i in 0..120 {
            let sx = ((i * 97) % width) as f32 + ((time * 10.0).sin() * 2.0) as f32;
            let sy = ((i * 53) % height) as f32 + ((time * 7.0).cos() * 2.0) as f32;
            d.draw_pixel(sx as i32, sy as i32, Color::new(200,200,255, 200));
        }

        // Draw orbits for each planet (not the sun)
        for (idx, b) in bodies.iter().enumerate() {
            if idx == 0 { continue; } // Skip sun
            
            let orbit_points = 100;
            for i in 0..orbit_points {
                let angle1 = (i as f32 / orbit_points as f32) * std::f32::consts::PI * 2.0;
                let angle2 = ((i + 1) as f32 / orbit_points as f32) * std::f32::consts::PI * 2.0;
                
                let ox1 = angle1.cos() * b.orbit_radius;
                let oz1 = angle1.sin() * b.orbit_radius;
                let ox2 = angle2.cos() * b.orbit_radius;
                let oz2 = angle2.sin() * b.orbit_radius;
                
                // Transform to camera space
                let mut c1 = Vector3::new(ox1 - cam_world_x, 0.0 - cam_y, oz1 - cam_world_z);
                let mut c2 = Vector3::new(ox2 - cam_world_x, 0.0 - cam_y, oz2 - cam_world_z);
                
                c1 = rotate_x(c1, -cam_pitch);
                c1 = rotate_y(c1, -cam_yaw);
                c2 = rotate_x(c2, -cam_pitch);
                c2 = rotate_y(c2, -cam_yaw);
                
                // Project to screen
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

        // Collect all triangles with depth for proper z-buffering
        let mut triangles: Vec<TriangleDepth> = Vec::new();

        // Render each body by transforming sphere triangles
        for (idx, b) in bodies.iter().enumerate() {
            // compute orbit position
            let angle = time * b.orbit_speed;
            let bx = angle.cos() * b.orbit_radius;
            let bz = angle.sin() * b.orbit_radius;

            // per-triangle render
            for i in (0..sphere_tris.len()).step_by(3) {
                if i + 2 >= sphere_tris.len() { break; }
                let mut v0 = sphere_tris[i];
                let mut v1 = sphere_tris[i+1];
                let mut v2 = sphere_tris[i+2];

                // scale
                v0.x *= b.size; v0.y *= b.size; v0.z *= b.size;
                v1.x *= b.size; v1.y *= b.size; v1.z *= b.size;
                v2.x *= b.size; v2.y *= b.size; v2.z *= b.size;

                // spin
                let spin_angle = time * b.spin_speed;
                v0 = rotate_y(v0, spin_angle);
                v1 = rotate_y(v1, spin_angle);
                v2 = rotate_y(v2, spin_angle);

                // translate to orbit position (ecliptic y=0)
                v0.x += bx; v0.z += bz;
                v1.x += bx; v1.z += bz;
                v2.x += bx; v2.z += bz;

                // view transform: move relative to camera anchor
                let mut c0 = Vector3::new(v0.x - cam_world_x, v0.y - cam_y, v0.z - cam_world_z);
                let mut c1 = Vector3::new(v1.x - cam_world_x, v1.y - cam_y, v1.z - cam_world_z);
                let mut c2 = Vector3::new(v2.x - cam_world_x, v2.y - cam_y, v2.z - cam_world_z);

                // rotate by camera pitch then yaw
                c0 = rotate_x(c0, -cam_pitch);
                c1 = rotate_x(c1, -cam_pitch);
                c2 = rotate_x(c2, -cam_pitch);
                c0 = rotate_y(c0, -cam_yaw);
                c1 = rotate_y(c1, -cam_yaw);
                c2 = rotate_y(c2, -cam_yaw);

                // backface culling
                let edge1 = Vector3::new(c1.x - c0.x, c1.y - c0.y, c1.z - c0.z);
                let edge2 = Vector3::new(c2.x - c0.x, c2.y - c0.y, c2.z - c0.z);
                let normal = Vector3::new(
                    edge1.y * edge2.z - edge1.z * edge2.y,
                    edge1.z * edge2.x - edge1.x * edge2.z,
                    edge1.x * edge2.y - edge1.y * edge2.x,
                );
                let center = Vector3::new((c0.x + c1.x + c2.x)/3.0, (c0.y + c1.y + c2.y)/3.0, (c0.z + c1.z + c2.z)/3.0);
                let view_dir = Vector3::new(-center.x, -center.y, -center.z);
                let dot = normal.x * view_dir.x + normal.y * view_dir.y + normal.z * view_dir.z;
                if dot >= 0.0 { continue; }

                // project
                let p0 = perspective_project(c0, width as f32, height as f32, 70.0);
                let p1 = perspective_project(c1, width as f32, height as f32, 70.0);
                let p2 = perspective_project(c2, width as f32, height as f32, 70.0);
                if p0.is_none() || p1.is_none() || p2.is_none() { continue; }
                let (x0,y0) = p0.unwrap(); let (x1,y1) = p1.unwrap(); let (x2,y2) = p2.unwrap();

                // fragment normal approximate
                let len = (normal.x*normal.x + normal.y*normal.y + normal.z*normal.z).sqrt();
                let n = if len > 0.0001 { Vector3::new(normal.x/len, normal.y/len, normal.z/len) } else { Vector3::new(0.0,1.0,0.0) };

                // compute fragment world center (for shader)
                let world_center = Vector3::new((v0.x+v1.x+v2.x)/3.0, (v0.y+v1.y+v2.y)/3.0, (v0.z+v1.z+v2.z)/3.0);
                let fragment = Fragment { world_position: world_center, normal: n };

                let color_v = match idx {
                    0 => {
                        // Sun: RUIDO FRACTAL (FBM intenso)
                        let uniforms = Uniforms::new(time, 1.0);
                        star_shader(&fragment, &uniforms)
                    },
                    1 => {
                        // Mercury: CELLULAR NOISE
                        cellular_planet_shader(&fragment, time, b.color)
                    },
                    2 => {
                        // Venus: SIMPLEX NOISE
                        simplex_planet_shader(&fragment, time, b.color)
                    },
                    3 => {
                        // Earth: VORONOI NOISE
                        voronoi_planet_shader(&fragment, time, b.color)
                    },
                    4 => {
                        // Mars: PERLIN NOISE
                        perlin_planet_shader(&fragment, time, b.color)
                    },
                    _ => {
                        // Goliath: Default shader mejorado
                        planet_shader(&fragment, time, b.color)
                    }
                };

                let fill = Color::new((color_v.x*255.0) as u8, (color_v.y*255.0) as u8, (color_v.z*255.0) as u8, 255);
                
                // Calculate average depth (Z) for sorting
                let avg_depth = (c0.z + c1.z + c2.z) / 3.0;
                
                // Add to triangles vector instead of drawing immediately
                triangles.push(TriangleDepth {
                    p0: (x0, y0),
                    p1: (x1, y1),
                    p2: (x2, y2),
                    color: fill,
                    depth: avg_depth,
                });
            }
        }

        // Sort triangles by depth (far to near) for proper rendering
        triangles.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap());
        
        // Draw all triangles in sorted order
        for tri in &triangles {
            d.draw_triangle(
                Vector2::new(tri.p0.0, tri.p0.1),
                Vector2::new(tri.p1.0, tri.p1.1),
                Vector2::new(tri.p2.0, tri.p2.1),
                tri.color
            );
        }

        // Render spaceship model enfrente de la cámara (fixed position in front of camera)
        if !ship_tris.is_empty() {
            let ship_scale = 0.05f32; // Even smaller scale for better view of space
            
            // Position ship in camera space (fixed in front of view)
            // This makes the ship always visible like a cockpit/first-person view
            let ship_cam_x = 0.0;      // centered horizontally
            let ship_cam_y = -0.4;     // lower position (cockpit bottom view)
            let ship_cam_z = 1.2;      // distance in front of camera
            
            for i in (0..ship_tris.len()).step_by(3) {
                if i + 2 >= ship_tris.len() { break; }
                let mut a = ship_tris[i];
                let mut b = ship_tris[i+1];
                let mut c = ship_tris[i+2];

                // scale model (smaller for first-person view)
                a.x *= ship_scale; a.y *= ship_scale; a.z *= ship_scale;
                b.x *= ship_scale; b.y *= ship_scale; b.z *= ship_scale;
                c.x *= ship_scale; c.y *= ship_scale; c.z *= ship_scale;

                // Rotate ship -90° horizontally (around Y axis) for proper orientation
                let rotation_y = -90.0_f32.to_radians(); // -90 degrees
                a = rotate_y(a, rotation_y);
                b = rotate_y(b, rotation_y);
                c = rotate_y(c, rotation_y);
                
                // translate to camera-relative position (fixed in front of view)
                a.x += ship_cam_x; a.y += ship_cam_y; a.z += ship_cam_z;
                b.x += ship_cam_x; b.y += ship_cam_y; b.z += ship_cam_z;
                c.x += ship_cam_x; c.y += ship_cam_y; c.z += ship_cam_z;

                // No view transform needed - ship is already in camera space
                let ca = a;
                let cb = b;
                let cc = c;

                // backface culling
                let e1 = Vector3::new(cb.x - ca.x, cb.y - ca.y, cb.z - ca.z);
                let e2 = Vector3::new(cc.x - ca.x, cc.y - ca.y, cc.z - ca.z);
                let n = Vector3::new(
                    e1.y * e2.z - e1.z * e2.y,
                    e1.z * e2.x - e1.x * e2.z,
                    e1.x * e2.y - e1.y * e2.x,
                );
                let center = Vector3::new((ca.x + cb.x + cc.x)/3.0, (ca.y + cb.y + cc.y)/3.0, (ca.z + cb.z + cc.z)/3.0);
                let view_dir = Vector3::new(-center.x, -center.y, -center.z);
                let dotp = n.x * view_dir.x + n.y * view_dir.y + n.z * view_dir.z;
                if dotp >= 0.0 { continue; }

                let pa = perspective_project(ca, width as f32, height as f32, 70.0);
                let pb = perspective_project(cb, width as f32, height as f32, 70.0);
                let pc = perspective_project(cc, width as f32, height as f32, 70.0);
                if pa.is_none() || pb.is_none() || pc.is_none() { continue; }
                let (ax,ay) = pa.unwrap(); let (bx,by) = pb.unwrap(); let (cx,cy) = pc.unwrap();
                
                // Calcular el centro del triángulo en espacio mundial para el shader
                let v0 = ship_tris[i];
                let v1 = ship_tris[i+1];
                let v2 = ship_tris[i+2];
                let world_center = Vector3::new(
                    (v0.x + v1.x + v2.x) / 3.0,
                    (v0.y + v1.y + v2.y) / 3.0,
                    (v0.z + v1.z + v2.z) / 3.0
                );
                
                // Normalizar la normal para el shader
                let normal_len = (n.x*n.x + n.y*n.y + n.z*n.z).sqrt();
                let normal = if normal_len > 0.0001 {
                    Vector3::new(n.x/normal_len, n.y/normal_len, n.z/normal_len)
                } else {
                    Vector3::new(0.0, 1.0, 0.0)
                };
                
                // Aplicar shader a la nave espacial
                let fragment = Fragment { world_position: world_center, normal };
                let shader_color = spaceship_shader(&fragment, time);
                
                // Si está en nitro, mezclar con cyan
                let final_color = if nitro_active {
                    Vector3::new(
                        shader_color.x * 0.3 + 0.4,
                        shader_color.y * 0.3 + 1.0,
                        shader_color.z * 0.3 + 1.0,
                    )
                } else {
                    shader_color
                };
                
                let ship_color = Color::new(
                    (final_color.x * 255.0).clamp(0.0, 255.0) as u8,
                    (final_color.y * 255.0).clamp(0.0, 255.0) as u8,
                    (final_color.z * 255.0).clamp(0.0, 255.0) as u8,
                    255
                );
                
                d.draw_triangle(Vector2::new(ax,ay), Vector2::new(bx,by), Vector2::new(cx,cy), ship_color);
            }
        } else {
            // simple marker if no model
            d.draw_circle((width/2) as i32, (height - 80) as i32, 12.0, Color::WHITE);
        }
        d.draw_text(&format!("Ship pos ({:.2},{:.2})", ship_x, ship_z), 20, height - 48, 18, Color::LIGHTGRAY);

    }
}
