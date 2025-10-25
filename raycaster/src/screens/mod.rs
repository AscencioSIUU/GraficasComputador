use raylib::prelude::*;

pub trait Screen {
    fn update(&mut self, rl: &RaylibHandle);
    fn next(&mut self) -> Option<Box<dyn Screen>>;
    fn draw_raylib(&mut self, d: &mut RaylibDrawHandle);
    fn as_any(&self) -> &dyn std::any::Any;
}

pub mod menu;
pub mod game;
pub mod win;
pub mod gameover;
