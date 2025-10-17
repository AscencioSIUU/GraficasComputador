use raylib::prelude::*;
use super::Screen;

#[derive(Default)]
pub struct PauseScreen {
    resume: bool,
    quit: bool,
}

impl Screen for PauseScreen {
    fn update(&mut self, rl: &RaylibHandle) {
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) { self.resume = true; }
        if rl.is_key_pressed(KeyboardKey::KEY_Q) { self.quit = true; }
    }
    fn draw_raylib(&mut self, d: &mut RaylibDrawHandle) {
        let w = d.get_screen_width();
        let h = d.get_screen_height();
        d.draw_rectangle(0, 0, w, h, Color::new(0,0,0,150));
        d.draw_text("Paused", w/2 - 60, h/2 - 40, 40, Color::WHITE);
        d.draw_text("Press ESC to resume, Q to quit to menu", w/2 - 180, h/2 + 10, 20, Color::LIGHTGRAY);
    }
    fn next(&mut self) -> Option<Box<dyn Screen>> {
        if self.quit {
            // go back to menu
            self.quit = false;
            Some(Box::new(crate::screens::menu::MenuScreen::default()))
        } else if self.resume {
            self.resume = false;
            // signal to pop pause: return None but GameScreen expects to handle resume via its own state
            None
        } else { None }
    }
}
