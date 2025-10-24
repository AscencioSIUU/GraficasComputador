use raylib::prelude::*;
use super::Screen;

pub struct WinScreen {
    return_to_menu: bool,
}

impl WinScreen {
    pub fn new() -> Self {
        Self { return_to_menu: false }
    }
}

impl Screen for WinScreen {
    fn update(&mut self, rl: &RaylibHandle) {
        // Press ENTER or ESC to return to menu
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            self.return_to_menu = true;
        }
    }

    fn next(&mut self) -> Option<Box<dyn Screen>> {
        if self.return_to_menu {
            self.return_to_menu = false;
            return Some(Box::new(crate::screens::menu::MenuScreen::default()));
        }
        None
    }

    fn draw_raylib(&mut self, d: &mut RaylibDrawHandle) {
        let w = d.get_screen_width();
        let h = d.get_screen_height();
        
        // Victory background
        d.clear_background(Color::new(10, 30, 10, 255));
        
        // "YOU WIN!" title
        let title = "YOU WIN!";
        let title_size = 80;
        let title_width = d.measure_text(title, title_size);
        d.draw_text(
            title,
            w / 2 - title_width / 2,
            h / 2 - 150,
            title_size,
            Color::GOLD
        );
        
        // Subtitle
        let subtitle = "Congratulations!";
        let subtitle_size = 40;
        let subtitle_width = d.measure_text(subtitle, subtitle_size);
        d.draw_text(
            subtitle,
            w / 2 - subtitle_width / 2,
            h / 2 - 60,
            subtitle_size,
            Color::GREEN
        );
        
        // Instructions
        let menu_text = "Press ENTER to return to menu";
        let menu_size = 25;
        let menu_width = d.measure_text(menu_text, menu_size);
        d.draw_text(
            menu_text,
            w / 2 - menu_width / 2,
            h / 2 + 80,
            menu_size,
            Color::LIGHTGRAY
        );
        
        // Trophy decoration
        d.draw_text("🏆", w / 2 - 30, h / 2 + 150, 60, Color::GOLD);
    }
}
