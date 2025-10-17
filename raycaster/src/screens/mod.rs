use raylib::prelude::*;

pub trait Screen {
    fn update(&mut self, _rl: &RaylibHandle) {}
    fn draw_raylib(&mut self, _d: &mut RaylibDrawHandle) {}
    fn next(&mut self) -> Option<Box<dyn Screen>> { None }
}

pub mod menu;
pub mod game;
pub mod win;
pub mod pause;
