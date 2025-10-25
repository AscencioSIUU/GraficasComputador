use raylib::prelude::*;
use super::Screen;

pub struct GameOverScreen {
    level_path: String, // Para reiniciar el nivel
    restart: bool,
    quit_to_menu: bool,
}

impl GameOverScreen {
    pub fn new(level_path: String) -> Self {
        Self {
            level_path,
            restart: false,
            quit_to_menu: false,
        }
    }
}

impl Screen for GameOverScreen {
    fn update(&mut self, rl: &raylib::RaylibHandle) {
        // Press ENTER to restart level
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            self.restart = true;
        }
        
        // Press ESC to go back to menu
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            self.quit_to_menu = true;
        }
    }

    fn next(&mut self) -> Option<Box<dyn Screen>> {
        if self.restart {
            self.restart = false;
            return Some(Box::new(crate::screens::game::GameScreen::new(self.level_path.clone())));
        }
        
        if self.quit_to_menu {
            self.quit_to_menu = false;
            return Some(Box::new(crate::screens::menu::MenuScreen::default()));
        }
        
        None
    }

    fn draw_raylib(&mut self, d: &mut raylib::prelude::RaylibDrawHandle) {
        let w = d.get_screen_width();
        let h = d.get_screen_height();
        
        // Dark red background overlay
        d.clear_background(Color::new(20, 0, 0, 255));
        
        // "GAME OVER" title
        let title = "GAME OVER";
        let title_size = 80;
        let title_width = d.measure_text(title, title_size);
        d.draw_text(
            title,
            w / 2 - title_width / 2,
            h / 2 - 150,
            title_size,
            Color::RED
        );
        
        // Subtitle
        let subtitle = "You have been defeated";
        let subtitle_size = 30;
        let subtitle_width = d.measure_text(subtitle, subtitle_size);
        d.draw_text(
            subtitle,
            w / 2 - subtitle_width / 2,
            h / 2 - 60,
            subtitle_size,
            Color::new(200, 50, 50, 255)
        );
        
        // Instructions
        let restart_text = "Press ENTER to restart level";
        let restart_size = 25;
        let restart_width = d.measure_text(restart_text, restart_size);
        d.draw_text(
            restart_text,
            w / 2 - restart_width / 2,
            h / 2 + 40,
            restart_size,
            Color::WHITE
        );
        
        let menu_text = "Press ESC to return to menu";
        let menu_size = 25;
        let menu_width = d.measure_text(menu_text, menu_size);
        d.draw_text(
            menu_text,
            w / 2 - menu_width / 2,
            h / 2 + 80,
            menu_size,
            Color::LIGHTGRAY
        );
        
        // Skull decoration (simple ASCII art)
        let skull_y = h / 2 + 150;
        d.draw_text("☠", w / 2 - 30, skull_y, 60, Color::new(150, 0, 0, 255));
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
