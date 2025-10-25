use raylib::prelude::*;
use super::Screen;
use crate::textures::TextureManager;

pub struct MenuScreen {
    selected: usize,
    start: bool,
    quit: bool,
    tex: Option<TextureManager>,
}

impl Default for MenuScreen {
    fn default() -> Self {
        let tex = TextureManager::from_assets().ok();
        Self { 
            selected: 0, 
            start: false,
            quit: false,
            tex,
        }
    }
}

impl Screen for MenuScreen {
    fn update(&mut self, rl: &raylib::RaylibHandle) {
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            self.selected = (self.selected + 1) % 4; // Ahora 4 opciones
        }
        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            self.selected = if self.selected == 0 { 3 } else { self.selected - 1 };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            if self.selected == 3 {
                self.quit = true;
            } else {
                self.start = true;
            }
        }
    }

    fn next(&mut self) -> Option<Box<dyn Screen>> {
        if self.start {
            let level = match self.selected {
                0 => "assets/maps/level1.txt",
                1 => "assets/maps/level2.txt",
                2 => "assets/maps/level3.txt",
                _ => "assets/maps/level1.txt",
            };
            return Some(Box::new(crate::screens::game::GameScreen::new(level.into())));
        }
        None
    }

    fn draw_raylib(&mut self, d: &mut raylib::prelude::RaylibDrawHandle) {
        let screen_w = d.get_screen_width();
        let screen_h = d.get_screen_height();
        
        // Draw background image if available (optimizado - cada 4 pixeles)
        if let Some(tex) = &self.tex {
            let menu_key = "menu";
            let (img_w, img_h) = tex.size_of(menu_key);
            
            let scale_w = screen_w as f32 / img_w as f32;
            let scale_h = screen_h as f32 / img_h as f32;
            let scale = scale_w.max(scale_h);
            
            let display_w = (img_w as f32 * scale) as i32;
            let display_h = (img_h as f32 * scale) as i32;
            
            let offset_x = (screen_w - display_w) / 2;
            let offset_y = (screen_h - display_h) / 2;
            
            let scale_x = img_w as f32 / display_w as f32;
            let scale_y = img_h as f32 / display_h as f32;
            
            // OPTIMIZADO: cada 4 pixeles
            for dy in (0..display_h).step_by(4) {
                let screen_y = offset_y + dy;
                if screen_y < 0 || screen_y >= screen_h { continue; }
                
                for dx in (0..display_w).step_by(4) {
                    let screen_x = offset_x + dx;
                    if screen_x < 0 || screen_x >= screen_w { continue; }
                    
                    let tx = (dx as f32 * scale_x) as usize;
                    let ty = (dy as f32 * scale_y) as usize;
                    
                    let color = tex.sample_at(menu_key, tx, ty);
                    d.draw_rectangle(screen_x, screen_y, 4, 4, color);
                }
            }
        } else {
            d.clear_background(Color::new(20, 20, 40, 255));
        }
        
        // Semi-transparent overlay
        d.draw_rectangle(0, 0, screen_w, screen_h, Color::new(0, 0, 0, 100));
        
        // Title
        let title = "RAYCASTER DUNGEON";
        let title_size = 60;
        let title_width = d.measure_text(title, title_size);
        d.draw_text(
            title,
            screen_w / 2 - title_width / 2,
            screen_h / 4,
            title_size,
            Color::GOLD
        );
        
        // Menu options
        let options = ["Level 1", "Level 2", "Level 3", "Exit"];
        let start_y = screen_h / 2;
        let spacing = 60;
        
        for (i, option) in options.iter().enumerate() {
            let size = 40;
            let text_width = d.measure_text(option, size);
            let y = start_y + (i as i32 * spacing);
            
            let color = if i == self.selected {
                Color::YELLOW
            } else {
                Color::WHITE
            };
            
            if i == self.selected {
                d.draw_text(">", screen_w / 2 - text_width / 2 - 40, y, size, Color::YELLOW);
            }
            
            d.draw_text(
                option,
                screen_w / 2 - text_width / 2,
                y,
                size,
                color
            );
        }
        
        // Instructions
        let instructions = "Use Arrow Keys to navigate, ENTER to select";
        let inst_size = 20;
        let inst_width = d.measure_text(instructions, inst_size);
        d.draw_text(
            instructions,
            screen_w / 2 - inst_width / 2,
            screen_h - 80,
            inst_size,
            Color::LIGHTGRAY
        );
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl MenuScreen {
    pub fn should_quit(&self) -> bool {
        self.quit
    }
}
