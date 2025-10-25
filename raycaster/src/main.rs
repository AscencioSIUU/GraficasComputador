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
        .size(1280, 720) // Ventana de tamaño fijo 1280x720
        .title("Rust Ray Caster")
        .build();
    
    // NO toggle fullscreen - mantener ventana normal
    rl.set_target_fps(60);
    rl.set_exit_key(None);

    let mut screen: Box<dyn Screen> = Box::new(MenuScreen::default());

    while !rl.window_should_close() {
        // Check if menu wants to quit
        if let Some(menu) = screen.as_any().downcast_ref::<MenuScreen>() {
            if menu.should_quit() {
                break;
            }
        }
        
        screen.update(&rl);

        let mut d = rl.begin_drawing(&th);
        d.clear_background(Color::BLACK);
        screen.draw_raylib(&mut d);
        drop(d);

        if let Some(next) = screen.next() {
            screen = next;
        }
    }
}
