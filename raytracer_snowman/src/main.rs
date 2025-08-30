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
    framebuffer.set_background_color(Color::new(10, 15, 40, 255)); // fondo azul oscuro

    // Colores
    let GOLD  = Color::new(229, 170, 75, 255);  // cara
    let MANE  = Color::new(180, 100, 30, 255);  // melena
    let BLACK = Color::BLACK;                   // ojos y nariz

    let z_plane: f32 = -6.0;

    let mut circle = |cx: f32, cy: f32, r: f32, c: Color| -> Sphere {
        Sphere::new(Vector3::new(cx, cy, z_plane), r, Material::new(c))
    };

    let mut objects: Vec<Sphere> = Vec::new();

    // Melena
    let mane_r_outer = 1.9f32;
    let mane_r_dot   = 0.65f32;
    let n_mane       = 18usize;
    for i in 0..n_mane {
        let t = (i as f32) * std::f32::consts::TAU / (n_mane as f32);
        let x = mane_r_outer * t.cos();
        let y = mane_r_outer * t.sin();
        objects.push(circle(x, y, mane_r_dot, MANE));
    }

    // Cabeza
    objects.push(circle(0.0, 0.0, 1.25, GOLD));

    // Ojos (negros)
    objects.push(circle(0.40, 0.25, 0.15, BLACK));
    objects.push(circle( 0.40, 0.25, 0.15, BLACK));

    // Nariz (negra, al centro)
    objects.push(circle(0.0, -0.25, 0.18, BLACK));

    let mut saved_png = false;

    while !handle.window_should_close() {
        framebuffer.clear();
        render(&mut framebuffer, &objects);

        if !saved_png {
            framebuffer.render_to_png("leon_simple.png");
            saved_png = true;
        }

        let texture = handle
            .load_texture_from_image(&raylib_thread, &framebuffer.color_buffer)
            .unwrap();

        let mut draw_handle = handle.begin_drawing(&raylib_thread);
        draw_handle.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}
