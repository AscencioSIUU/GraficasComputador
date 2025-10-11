mod framebuffer;
mod raytracer;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use raytracer::{
    Material as RtMaterial, Sphere, Triangle, render,
    cube_local_triangles_minecraft,
    ImageTexture,
    rotate_euler, // <- usamos Euler libre
};

fn main() {
    let (mut rl, th) = raylib::init()
        .size(800, 800)
        .title("Minecraft-style Textured Cube (free controls)")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut fb = Framebuffer::new(800, 800, Color::new(20, 24, 40, 255));
    fb.set_background_color(Color::new(20, 24, 40, 255));

    // ---- movable transform state ----
    let mut center = Vector3::new(0.0, 0.0, -8.0);
    let half   = 1.8_f32;

    // Euler angles (radians)
    let mut yaw:   f32 = 0.0;  // rotate around Y
    let mut pitch: f32 = 0.0;  // rotate around X
    let mut roll:  f32 = 0.0;  // rotate around Z

    // Textures & materials (top / side / bottom)
    let tex_top    = ImageTexture::from_file_or_checker("assets/grass_top.png");
    let tex_side   = ImageTexture::from_file_or_checker("assets/grass_side.png");
    let tex_bottom = ImageTexture::from_file_or_checker("assets/dirt.png");

    let mat_top    = RtMaterial::with_texture(Color::WHITE, tex_top);
    let mat_side   = RtMaterial::with_texture(Color::WHITE, tex_side);
    let mat_bottom = RtMaterial::with_texture(Color::WHITE, tex_bottom);

    // Local triangles (stay constant)
    let local_tris = cube_local_triangles_minecraft(half, &mat_top, &mat_side, &mat_bottom);

    // Optional ground
    let ground = Sphere::new(
        Vector3::new(0.0, -100.0, -8.0),
        98.0,
        RtMaterial::new(Color::new(110,130,160,255))
    );
    let spheres: Vec<Sphere> = vec![ground];

    while !rl.window_should_close() {
        // ---- input: move/rotate ----
        let base_move = 0.10_f32;
        let base_rot  = 0.02_f32;
        let fast_mul  = if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) { 4.0 } else { 1.0 };
        let ms = base_move * fast_mul;
        let rs = base_rot  * fast_mul;

        // Translation (WASD + Q/E)
        if rl.is_key_down(KeyboardKey::KEY_W) { center.z -= ms; } // forward (camera looks -Z)
        if rl.is_key_down(KeyboardKey::KEY_S) { center.z += ms; } // backward
        if rl.is_key_down(KeyboardKey::KEY_A) { center.x -= ms; } // left
        if rl.is_key_down(KeyboardKey::KEY_D) { center.x += ms; } // right
        if rl.is_key_down(KeyboardKey::KEY_Q) { center.y += ms; } // up
        if rl.is_key_down(KeyboardKey::KEY_E) { center.y -= ms; } // down

        // Rotation (arrows + Z/X)
        if rl.is_key_down(KeyboardKey::KEY_LEFT)  { yaw   -= rs; }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { yaw   += rs; }
        if rl.is_key_down(KeyboardKey::KEY_UP)    { pitch -= rs; }
        if rl.is_key_down(KeyboardKey::KEY_DOWN)  { pitch += rs; }
        if rl.is_key_down(KeyboardKey::KEY_Z)     { roll  -= rs; }
        if rl.is_key_down(KeyboardKey::KEY_X)     { roll  += rs; }

        // Reset
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            center = Vector3::new(0.0, 0.0, -8.0);
            yaw = 0.0; pitch = 0.0; roll = 0.0;
        }

        // ---- build world-space triangles using Euler rotation ----
        let mut tris: Vec<Triangle> = Vec::with_capacity(local_tris.len());
        for t in &local_tris {
            let ar = rotate_euler(t.v0, yaw, pitch, roll) + center;
            let br = rotate_euler(t.v1, yaw, pitch, roll) + center;
            let cr = rotate_euler(t.v2, yaw, pitch, roll) + center;

            tris.push(Triangle {
                v0: ar, v1: br, v2: cr,
                uv0: t.uv0, uv1: t.uv1, uv2: t.uv2,
                mat: t.mat.clone(),
            });
        }

        // ---- render & present ----
        fb.clear();
        render(&mut fb, &spheres, &tris);

        let screen_tex = rl.load_texture_from_image(&th, &fb.color_buffer).unwrap();
        let mut d = rl.begin_drawing(&th);
        d.draw_texture(&screen_tex, 0, 0, Color::WHITE);
    }
}
