use raylib::prelude::*;
mod obj;
mod shader;

use obj::Obj;
use raylib::math::{Vector2, Vector3};
use shader::{Fragment, Uniforms, fragment_shader};

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
    let (width, height) = (1024, 768);
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Spaceship OBJ viewer (3D camera)")
        .build();

    let mut load_error_msg: Option<String> = None;
    let mut vertex_array: Vec<Vector3> = Vec::new();

    let try_paths = vec!["spaceship.obj"];
    let mut loaded = false;
    for p in try_paths {
        match Obj::load(p) {
            Ok(m) => { vertex_array = m.get_vertex_array(); loaded = true; break; }
            Err(e) => { load_error_msg = Some(format!("{} -> {:?}", p, e)); }
        }
    }

    if !loaded {
        if let Ok(entries) = std::fs::read_dir(".") {
            for e in entries.flatten() {
                let path = e.path();
                if let Some(ext) = path.extension() {
                    if ext == "obj" {
                        if let Some(pstr) = path.to_str() {
                            match Obj::load(pstr) {
                                Ok(m) => { vertex_array = m.get_vertex_array(); loaded = true; break; }
                                Err(e) => { load_error_msg = Some(format!("{} -> {:?}", pstr, e)); }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut cam_x: f32 = 0.0;
    let mut cam_y: f32 = 0.0;
    let mut cam_z: f32 = -3.0;
    let mut cam_yaw: f32 = 0.0;
    let mut cam_pitch: f32 = 0.0;
    let move_speed: f32 = 0.05;
    let rot_speed: f32 = 0.02;
    let mut wireframe = false;
    let mut fov: f32 = 60.0;
    let mut time: f32 = 0.0;

    let mut centroid = Vector3::new(0.0, 0.0, 0.0);
    if !vertex_array.is_empty() {
        for v in &vertex_array { centroid.x += v.x; centroid.y += v.y; centroid.z += v.z; }
        let n = vertex_array.len() as f32;
        centroid.x /= n; centroid.y /= n; centroid.z /= n;
    }

    while !rl.window_should_close() {
        time += rl.get_frame_time();
        
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_RIGHT) { cam_yaw += rot_speed; }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_LEFT) { cam_yaw -= rot_speed; }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_UP) { cam_pitch += rot_speed; }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_DOWN) { cam_pitch -= rot_speed; }

        // Movement relative to camera orientation: WASD moves on XZ plane
        let forward = Vector3::new(cam_yaw.cos(), 0.0, cam_yaw.sin());
        let right = Vector3::new(-forward.z, 0.0, forward.x);
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_W) {
            cam_x += forward.x * move_speed;
            cam_z += forward.z * move_speed;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_S) {
            cam_x -= forward.x * move_speed;
            cam_z -= forward.z * move_speed;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_A) {
            cam_x -= right.x * move_speed;
            cam_z -= right.z * move_speed;
        }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_D) {
            cam_x += right.x * move_speed;
            cam_z += right.z * move_speed;
        }
        // Up / down
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_Q) { cam_y += move_speed; }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_E) { cam_y -= move_speed; }

        if rl.is_key_pressed(raylib::consts::KeyboardKey::KEY_SPACE) { wireframe = !wireframe; }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_KP_ADD) || rl.is_key_down(raylib::consts::KeyboardKey::KEY_EQUAL) { fov = (fov - 0.5).max(15.0); }
        if rl.is_key_down(raylib::consts::KeyboardKey::KEY_KP_SUBTRACT) || rl.is_key_down(raylib::consts::KeyboardKey::KEY_MINUS) { fov = (fov + 0.5).min(120.0); }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        d.draw_text(&format!("Pos: ({:.2},{:.2},{:.2})  Yaw:{:.2} Pitch:{:.2}  FOV:{:.1}", 
            cam_x, cam_y, cam_z, cam_yaw, cam_pitch, fov), 10, 10, 20, Color::WHITE);
        d.draw_text("W/S/A/D: move  Q/E: up/down  Arrows: rotate  +/-: FOV  Space: wireframe", 10, 30, 18, Color::WHITE);

        if !loaded {
            let msg = load_error_msg.clone().unwrap_or_else(|| "No .obj models found in current directory.".to_string());
            d.draw_text("ERROR: failed to load any OBJ", 100, 140, 30, Color::RED);
            d.draw_text(&msg, 100, 180, 20, Color::WHITE);
            d.draw_text("Place an .obj in the project folder or run from its folder. Close window to exit.", 100, 220, 20, Color::WHITE);
            continue;
        }

        for i in (0..vertex_array.len()).step_by(3) {
            if i + 2 >= vertex_array.len() { break; }
            let va = vertex_array[i];
            let vb = vertex_array[i + 1];
            let vc = vertex_array[i + 2];

            let ma = Vector3::new(va.x - centroid.x, va.y - centroid.y, va.z - centroid.z);
            let mb = Vector3::new(vb.x - centroid.x, vb.y - centroid.y, vb.z - centroid.z);
            let mc = Vector3::new(vc.x - centroid.x, vc.y - centroid.y, vc.z - centroid.z);

            let mut ca = Vector3::new(ma.x - cam_x, ma.y - cam_y, ma.z - cam_z);
            let mut cb = Vector3::new(mb.x - cam_x, mb.y - cam_y, mb.z - cam_z);
            let mut cc = Vector3::new(mc.x - cam_x, mc.y - cam_y, mc.z - cam_z);

            ca = rotate_x(ca, -cam_pitch);
            ca = rotate_y(ca, -cam_yaw);
            cb = rotate_x(cb, -cam_pitch);
            cb = rotate_y(cb, -cam_yaw);
            cc = rotate_x(cc, -cam_pitch);
            cc = rotate_y(cc, -cam_yaw);

            let edge1 = Vector3::new(cb.x - ca.x, cb.y - ca.y, cb.z - ca.z);
            let edge2 = Vector3::new(cc.x - ca.x, cc.y - ca.y, cc.z - ca.z);
            let normal = Vector3::new(
                edge1.y * edge2.z - edge1.z * edge2.y,
                edge1.z * edge2.x - edge1.x * edge2.z,
                edge1.x * edge2.y - edge1.y * edge2.x
            );
            
            let normal_len = (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z).sqrt();
            
            if normal_len < 0.0001 { continue; }
            
            let normalized_normal = Vector3::new(
                normal.x / normal_len, 
                normal.y / normal_len, 
                normal.z / normal_len
            );
            
            let center = Vector3::new(
                (ca.x + cb.x + cc.x) / 3.0,
                (ca.y + cb.y + cc.y) / 3.0,
                (ca.z + cb.z + cc.z) / 3.0
            );
            let view_dir = Vector3::new(-center.x, -center.y, -center.z);
            let view_len = (view_dir.x * view_dir.x + view_dir.y * view_dir.y + view_dir.z * view_dir.z).sqrt();
            let normalized_view = Vector3::new(view_dir.x / view_len, view_dir.y / view_len, view_dir.z / view_len);
            
            let dot = normalized_normal.x * normalized_view.x + 
                      normalized_normal.y * normalized_view.y + 
                      normalized_normal.z * normalized_view.z;
            
            if dot <= 0.01 { continue; }

            let pa = perspective_project(ca, width as f32, height as f32, fov);
            let pb = perspective_project(cb, width as f32, height as f32, fov);
            let pc = perspective_project(cc, width as f32, height as f32, fov);

            if pa.is_none() || pb.is_none() || pc.is_none() { continue; }
            let pa = pa.unwrap(); let pb = pb.unwrap(); let pc = pc.unwrap();

            if wireframe {
                d.draw_line_v(pa, pb, Color::PURPLE);
                d.draw_line_v(pb, pc, Color::PURPLE);
                d.draw_line_v(pc, pa, Color::PURPLE);
            } else {
                // Calcular el centro del triángulo en espacio mundial
                let world_center = Vector3::new(
                    (ma.x + mb.x + mc.x) / 3.0,
                    (ma.y + mb.y + mc.y) / 3.0,
                    (ma.z + mb.z + mc.z) / 3.0
                );
                
                // Aplicar shader
                let uniforms = Uniforms::new(time);
                let fragment = Fragment {
                    world_position: world_center,
                    color: Vector3::new(1.0, 1.0, 1.0),
                };
                let shader_color = fragment_shader(&fragment, &uniforms);
                
                // Convertir a Color de raylib
                let fill_color = Color::new(
                    (shader_color.x * 255.0).clamp(0.0, 255.0) as u8,
                    (shader_color.y * 255.0).clamp(0.0, 255.0) as u8,
                    (shader_color.z * 255.0).clamp(0.0, 255.0) as u8,
                    255
                );
                
                // Dibujar triángulo con color del shader
                d.draw_triangle(pa, pb, pc, fill_color);
                d.draw_triangle(pa, pc, pb, fill_color);
                
                // Dibujar aristas moradas
                d.draw_line_v(pa, pb, Color::PURPLE);
                d.draw_line_v(pb, pc, Color::PURPLE);
                d.draw_line_v(pc, pa, Color::PURPLE);
            }
        }
    }
}