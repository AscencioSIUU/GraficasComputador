#![allow(warnings)]

mod framebuffer;
mod raytracer;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use raytracer::{Material, Sphere, render};

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 800;

const FREMEBUFFER_WIDTH: i32 = WINDOW_WIDTH;
const FRAMEBUFFER_HEIGHT: i32 = WINDOW_HEIGHT;

fn main() {
    game_loop();
}

fn game_loop() {
    let (mut handle, raylib_thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Leon Raytracer")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(FREMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT, Color::WHITE);
    framebuffer.set_background_color(Color::new(10, 15, 40, 255)); // dark blue bg

    // Colors
    let GOLD  = Color::new(229, 170, 75, 255); // face
    let GOLD2  = Color::new(231, 171, 121, 255); // body
    let GOLD3 = Color::new(230, 112, 40, 255); // body
    let MANE1 = Color::new(180, 100, 30, 255); // mane
    let MANE2 = Color::new(230, 140, 30, 255); // mane
    let BLACK = Color::BLACK;                  // eyes/nose

    let z_plane: f32 = -6.0;

    let mut circle = |cx: f32, cy: f32, r: f32, c: Color| -> Sphere {
        Sphere::new(Vector3::new(cx, cy, z_plane), r, Material::new(c))
    };

    let mut objects: Vec<Sphere> = Vec::new();

    // Eyes
    objects.push(circle(-0.40,  0.25, 0.15, BLACK));
    objects.push(circle( 0.40,  0.25, 0.15, BLACK));
    // Nose
    objects.push(circle( 0.00, -0.25, 0.30, BLACK));

    // Mane (alternating colors)
    let mane_r_outer = 1.9_f32;
    let mane_r_dot   = 0.65_f32;
    let n_mane       = 18_usize;
    for i in 0..n_mane {
        let t = (i as f32) * std::f32::consts::TAU / (n_mane as f32);
        let x = mane_r_outer * t.cos();
        let y = mane_r_outer * t.sin();
        objects.push(circle(x, y, mane_r_dot, if i % 2 == 0 { MANE1 } else { MANE2 }));
    }

    // Head
    objects.push(circle(0.0, 0.0, 1.25, GOLD));

    // Body
    objects.push(circle(0.0, -2.0, 1.8, GOLD2));

    // Optional: soften body sides
    objects.push(circle(-1.4, -2.3, 0.7, GOLD3));
    objects.push(circle( 1.4, -2.3, 0.7, GOLD3));

    // Legs (helper)
    let mut add_leg = |x: f32| {
        objects.push(circle(x, -3.2, 0.55, GOLD3)); // upper leg
        objects.push(circle(x, -3.9, 0.45, GOLD3)); // paw
    };
    add_leg(-0.9);
    add_leg( 0.9);

    let mut saved_png = false;

    while !handle.window_should_close() {
        framebuffer.clear();
        render(&mut framebuffer, &objects);

        if !saved_png {
            framebuffer.render_to_png("leon.png");
            saved_png = true;
        }

        let texture = handle
            .load_texture_from_image(&raylib_thread, &framebuffer.color_buffer)
            .unwrap();

        let mut d = handle.begin_drawing(&raylib_thread);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}
