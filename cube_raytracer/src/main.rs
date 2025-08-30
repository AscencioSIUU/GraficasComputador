mod framebuffer;
mod raytracer;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use raytracer::{
    Material as RtMaterial, Sphere, Triangle, render,
    cube_local_triangles, rotate_y,
};

fn main() {
    let (mut rl, th) = raylib::init()
        .size(800, 800)
        .title("Smooth-faced Cube (diffuse)")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut fb = Framebuffer::new(800, 800, Color::new(20, 24, 40, 255));
    fb.set_background_color(Color::new(20, 24, 40, 255));

    // Scene params
    let center   = Vector3::new(0.0, 0.0, -8.0);
    let half     = 1.8_f32;                  // cube half-extent
    let cube_col = Color::new(230, 200, 80, 255);
    let mat      = RtMaterial::new(cube_col);

    // Precompute local triangles of a unit cube at origin
    let local_tris = cube_local_triangles(half);

    // Optional ground
    let ground = Sphere::new(Vector3::new(0.0, -100.0, -8.0), 98.0, RtMaterial::new(Color::new(110,130,160,255)));
    let mut spheres: Vec<Sphere> = vec![ground];

    let mut angle: f32 = 0.0;
    // Create texture once (optional)
    let mut texture = rl.load_texture_from_image(&th, &fb.color_buffer).unwrap();

    while !rl.window_should_close() {
        angle += 0.02;

        // Build world-space triangles by rotating + translating each vertex
        let mut tris: Vec<Triangle> = Vec::with_capacity(local_tris.len());
        for [a, b, c] in &local_tris {
            let ar = rotate_y(*a, angle);
            let br = rotate_y(*b, angle);
            let cr = rotate_y(*c, angle);
            tris.push(Triangle::new(center + ar, center + br, center + cr, mat));
        }

        fb.clear();
        render(&mut fb, &spheres, &tris);

        // (If your raylib binding has a texture update, use it; else recreate)
        texture = rl.load_texture_from_image(&th, &fb.color_buffer).unwrap();

        let mut d = rl.begin_drawing(&th);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}
