use raylib::prelude::*;
use super::Screen;
use crate::{
    player::Player,
    maze::{Maze, load_maze},
    ui, input,
    textures::TextureManager,
    raycaster,
    audio,
 };

pub struct GameScreen {
    maze: Maze,
    player: Player,
    level_path: String,
    tex: TextureManager,
    zbuffer: Vec<f32>,
    music: Option<raylib::prelude::Music<'static>>,
    muted: bool,
    music_volume: f32,
    render_scale: usize,
    scale_cooldown: i32, // frames until next auto-scale decision
    paused: bool,
    quit_to_menu: bool, // flag to return to menu from pause
    coins_collected: i32, // contador de monedas recolectadas
}

impl GameScreen {
    pub fn new(level_path: String) -> Self {
        let maze = load_maze(&level_path);
        let player = Player::new(Vector2::new(96.0, 96.0));
        let tex = TextureManager::from_assets().expect("load textures");
    let zbuffer = vec![f32::INFINITY; 2048]; // will be resized first frame
    // Start with higher-quality scale=4 as requested (balance quality + perf)
    let render_scale = 4usize;
    // Give an initial cooldown so the auto-adjust doesn't immediately change the scale
    let scale_cooldown = 120i32; // ~2 seconds at 60fps
        // load music stream from global audio device (if possible)
        let mut music = audio::load_music("assets/audios/dungeon_delver.mp3");
        let default_volume = 0.5;
        if let Some(m) = music.as_ref() {
            m.set_volume(default_volume);
        }
        let paused = false;
        let quit_to_menu = false;
        let coins_collected = 0;

        if let Some(mut m) = music {
            audio::play_music(&m);
            Self { maze, player, level_path, tex, zbuffer, music: Some(m), muted: false, music_volume: default_volume, render_scale, scale_cooldown, paused, quit_to_menu, coins_collected }
        } else {
            Self { maze, player, level_path, tex, zbuffer, music: None, muted: false, music_volume: default_volume, render_scale, scale_cooldown, paused, quit_to_menu, coins_collected }
        }
    }
}

impl Screen for GameScreen {
    fn update(&mut self, rl: &RaylibHandle) {
        // Toggle pause with ESC
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            self.paused = !self.paused;
        }

        // Handle quit to menu when paused
        if self.paused && rl.is_key_pressed(KeyboardKey::KEY_Q) {
            self.quit_to_menu = true;
        }

        // Only process game input when not paused
        if !self.paused {
            input::process(rl, &mut self.player, &self.maze);
            
            // Check for coin collection
            let block_size = maze::block_size() as f32;
            let tile_x = (self.player.pos.x / block_size) as usize;
            let tile_y = (self.player.pos.y / block_size) as usize;
            
            // Check if player is on a coin tile
            if tile_y < self.maze.len() && tile_x < self.maze[tile_y].len() {
                let tile_char = self.maze[tile_y][tile_x];
                if tile_char == 'X' || tile_char == 'x' {
                    // Collect the coin
                    self.maze[tile_y][tile_x] = ' ';
                    self.coins_collected += 1;
                }
            }
            
            // manual override for render scale: Z = finer (lower scale), X = coarser (higher scale)
            if rl.is_key_pressed(KeyboardKey::KEY_Z) {
                if self.render_scale > 1 { self.render_scale -= 1; }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_X) {
                if self.render_scale < 6 { self.render_scale += 1; }
            }

            // decrement cooldown used for automatic scaling decisions
            if self.scale_cooldown > 0 { self.scale_cooldown -= 1; }
            
            // Toggle mute with M
            if rl.is_key_pressed(KeyboardKey::KEY_M) {
                self.muted = !self.muted;
                if let Some(m) = &self.music {
                    if self.muted { m.set_volume(0.0); } else { m.set_volume(self.music_volume); }
                }
            }
        }

        // Always update music even when paused
        if let Some(m) = &self.music {
            audio::update_music(m);
            // explicit loop: if stream stopped, replay
            if !m.is_stream_playing() {
                m.play_stream();
            }
        }
    }

    fn next(&mut self) -> Option<Box<dyn Screen>> {
        if self.quit_to_menu {
            self.quit_to_menu = false;
            return Some(Box::new(crate::screens::menu::MenuScreen::default()));
        }
        None
    }

    fn draw_raylib(&mut self, d: &mut RaylibDrawHandle) {
    // make sure zbuffer matches current screen width divided by render_scale
    let w = d.get_screen_width().max(1) as usize;
    let scaled_w = (w + self.render_scale - 1) / self.render_scale; // ceil division
    if self.zbuffer.len() != scaled_w { self.zbuffer.resize(scaled_w, f32::INFINITY); }

    // 3D world (pass render_scale to let renderer draw blocks instead of single pixels)
    raycaster::draw_world(d, &self.maze, &self.player, &mut self.zbuffer, &self.tex, self.render_scale);
    
    // Draw coins as sprites
    raycaster::draw_coins(d, &self.maze, &self.player, &self.zbuffer, &self.tex, self.render_scale);

    // HUD (FPS/HP/minimap)
    let fps_i = d.get_fps();
    d.draw_text(&format!("FPS: {}", fps_i), 10, 10, 20, Color::LIGHTGRAY);
    let fps = fps_i as f32;
    // music status
    let music_status = if self.muted { "Muted".to_string() } else { format!("On {}%", (self.music_volume * 100.0) as i32) };
    d.draw_text(&format!("Music: {} (M to toggle)", music_status), 10, 30, 20, Color::LIGHTGRAY);
    d.draw_text(&format!("Render scale: {} (Z/X to change)", self.render_scale), 10, 50, 20, Color::LIGHTGRAY);
    // Coin counter
    d.draw_text(&format!("Coins: {}", self.coins_collected), 10, 70, 20, Color::GOLD);
    // Auto-adjust render scale to try to reach ~60 FPS (cooldown prevents rapid thrash)
    if self.scale_cooldown <= 0 {
        if fps < 55.0 && self.render_scale < 6 {
            self.render_scale += 1;
            self.scale_cooldown = 120; // wait ~2 seconds before next change
        } else if fps > 62.0 && self.render_scale > 1 {
            self.render_scale -= 1;
            self.scale_cooldown = 120;
        }
    }
    ui::draw_minimap(d, &self.maze, &self.player);
    
    // Draw pause overlay if paused
    if self.paused {
        let w = d.get_screen_width();
        let h = d.get_screen_height();
        d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, 150));
        d.draw_text("Paused", w/2 - 60, h/2 - 40, 40, Color::WHITE);
        d.draw_text("Press ESC to resume, Q to quit to menu", w/2 - 180, h/2 + 10, 20, Color::LIGHTGRAY);
    }
    }
}

impl Drop for GameScreen {
    fn drop(&mut self) {
        if let Some(m) = &self.music { audio::stop_music(m); }
    }
}
