use raylib::prelude::*;
use super::Screen;
use crate::screens::game::GameScreen;

#[derive(Default)]
pub struct MenuScreen {
    selected: usize,
    levels: Vec<String>,
    go_play: bool,
}

impl MenuScreen {
    fn load_levels() -> Vec<String> {
        vec![
            "assets/maps/level1.txt".into(),
            "assets/maps/level2.txt".into(),
        ]
    }
}

impl Screen for MenuScreen {
    fn update(&mut self, rl: &RaylibHandle) {
        if self.levels.is_empty() { self.levels = Self::load_levels(); }
        if rl.is_key_pressed(KeyboardKey::KEY_UP)   { self.selected = self.selected.saturating_sub(1); }
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) { self.selected = (self.selected + 1).min(self.levels.len().saturating_sub(1)); }
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) { self.go_play = true; }
    }
    fn draw_raylib(&mut self, d: &mut RaylibDrawHandle) {
        d.draw_text("Rust Ray Caster", 40, 60, 40, Color::WHITE);
        d.draw_text("Press ENTER to play", 40, 120, 20, Color::LIGHTGRAY);
        for (i, lvl) in self.levels.iter().enumerate() {
            let y = 180 + i as i32 * 24;
            let marker = if i == self.selected { ">" } else { " " };
            d.draw_text(&format!("{marker} {lvl}"), 60, y, 20, Color::SKYBLUE);
        }
    }
    fn next(&mut self) -> Option<Box<dyn Screen>> {
        if self.go_play {
            let path = self.levels.get(self.selected).cloned().unwrap_or_else(|| "assets/maps/level1.txt".into());
            self.go_play = false;
            Some(Box::new(GameScreen::new(path)))
        } else { None }
    }
}
