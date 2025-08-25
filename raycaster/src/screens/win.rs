use raylib::prelude::*;
use super::Screen;

pub struct WinScreen;
impl WinScreen { pub fn new() -> Self { Self } }

impl Screen for WinScreen {
    fn draw_raylib(&mut self, d: &mut RaylibDrawHandle) {
        d.draw_text("You Win!", 40, 60, 40, Color::GREEN);
        d.draw_text("Press ESC to exit", 40, 120, 20, Color::LIGHTGRAY);
    }
}
