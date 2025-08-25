mod framebuffer;
mod math;
mod input;
mod player;
mod maze;
mod raycaster;
mod textures;
mod sprites;
mod audio;
mod ui;
mod config;
mod screens; // ← SOLO UNA VEZ

use raylib::prelude::*;
use screens::{Screen, menu::MenuScreen};

fn main() {
    let (mut rl, th) = raylib::init()
        .size(1280, 720)
        .title("Rust Ray Caster")
        .build();
    rl.set_target_fps(60);

    let mut screen: Box<dyn Screen> = Box::new(MenuScreen::default());

    while !rl.window_should_close() {
        screen.update(&rl);

        let mut d = rl.begin_drawing(&th);
        d.clear_background(Color::BLACK);
        screen.draw_raylib(&mut d); // ← ESTA ES LA FIRMA CORRECTA
        drop(d);

        if let Some(next) = screen.next() {
            screen = next;
        }
    }
}
