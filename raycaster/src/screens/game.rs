use raylib::prelude::*;
use super::Screen;
use crate::{
    player::Player,
    maze::{Maze, load_maze},
    ui, input,
    textures::TextureManager,
    raycaster,
    audio,
    sprites::EnemySystem,
};

pub struct GameScreen {
    maze: Maze,
    player: Player,
    enemies: EnemySystem,
    level_path: String,
    tex: TextureManager,
    zbuffer: Vec<f32>,
    fog_of_war: crate::maze::FogOfWar, // Nuevo campo
    music: Option<raylib::prelude::Music<'static>>,
    muted: bool,
    music_volume: f32,
    render_scale: usize,
    scale_cooldown: i32, // frames until next auto-scale decision
    paused: bool,
    quit_to_menu: bool, // flag to return to menu from pause
    coins_collected: i32, // contador de monedas recolectadas
    advance_to: Option<String>, // path of next level or "WIN"
    game_over: bool, // nuevo flag
}

impl GameScreen {
    pub fn new(level_path: String) -> Self {
        let maze = load_maze(&level_path);
        let player = Player::new(Vector2::new(96.0, 96.0));
        let tex = TextureManager::from_assets().expect("load textures");
        
        // Crear fog of war
        let fog_of_war = crate::maze::FogOfWar::new(
            maze[0].len(),
            maze.len()
        );
        
        // Spawn enemies - 20 or more per level
        let mut enemies = EnemySystem::new();
        let enemy_count = if level_path.contains("level1") {
            20
        } else if level_path.contains("level2") {
            25
        } else {
            30 // level 3 has most enemies
        };
        enemies.spawn_from_maze(&maze, enemy_count);
        
        let zbuffer = vec![f32::INFINITY; 2048];
        // Empezar con render_scale 3 para balance entre calidad y FPS
        let render_scale = 3usize; // Balance: no muy pixeleado pero buen FPS
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
        let advance_to = None;
        let game_over = false;

        if let Some(mut m) = music {
            audio::play_music(&m);
            Self { 
                maze, 
                player, 
                enemies, 
                level_path, 
                tex, 
                zbuffer, 
                fog_of_war, // Agregar aquí
                music: Some(m), 
                muted: false, 
                music_volume: default_volume, 
                render_scale, 
                scale_cooldown, 
                paused, 
                quit_to_menu, 
                coins_collected, 
                advance_to, 
                game_over 
            }
        } else {
            Self { 
                maze, 
                player, 
                enemies, 
                level_path, 
                tex, 
                zbuffer, 
                fog_of_war, // Agregar aquí también
                music: None, 
                muted: false, 
                music_volume: default_volume, 
                render_scale, 
                scale_cooldown, 
                paused, 
                quit_to_menu, 
                coins_collected, 
                advance_to, 
                game_over 
            }
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

        // Only process game input when not paused and alive
        if !self.paused && self.player.is_alive() {
            input::process(rl, &mut self.player, &self.maze);
            
            // Actualizar fog of war
            self.fog_of_war.update(self.player.pos, &self.maze, 400.0); // Radio de visión 400 pixeles
            
            // Update shot effect timer
            self.player.update_shot_effect();
            
            // Update enemies (now they can shoot at player)
            self.enemies.update(&mut self.player, &self.maze);
            
            // Shoot enemies with SPACE
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                if let Some(enemy) = self.enemies.get_enemy_at_crosshair(&self.player, 500.0) {
                    enemy.take_damage(34); // Kill in 3 shots
                    self.player.trigger_shot_effect(); // Activar efecto visual
                }
            }
            
            // Check for coin collection
            let block_size = crate::maze::block_size() as f32;
            let tile_x = (self.player.pos.x / block_size) as usize;
            let tile_y = (self.player.pos.y / block_size) as usize;
            
            // Check if player is on a coin tile
            if tile_y < self.maze.len() && tile_x < self.maze[tile_y].len() {
                let tile_char = self.maze[tile_y][tile_x];
                if tile_char == 'X' || tile_char == 'x' {
                    // Collect the coin
                    self.maze[tile_y][tile_x] = ' ';
                    self.coins_collected += 1;
                    // If player collected 10 coins, prepare to advance
                    if self.coins_collected >= 10 && self.advance_to.is_none() {
                        // Determine next level path (simple sequence)
                        let next = if self.level_path.contains("level1") {
                            "assets/maps/level2.txt".into()
                        } else if self.level_path.contains("level2") {
                            "assets/maps/level3.txt".into()
                        } else {
                            "WIN".into()
                        };
                        self.advance_to = Some(next);
                    }
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
        
        // Check if player died
        if !self.player.is_alive() && !self.game_over {
            self.game_over = true;
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
        
        // Transition to Game Over screen if player died
        if self.game_over {
            return Some(Box::new(crate::screens::gameover::GameOverScreen::new(self.level_path.clone())));
        }
        
        // Advance to next level if flagged
        if let Some(next) = self.advance_to.take() {
            if next == "WIN" {
                return Some(Box::new(crate::screens::win::WinScreen::new()));
            } else {
                return Some(Box::new(crate::screens::game::GameScreen::new(next)));
            }
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
        
        // Draw enemies as sprites (solo los visibles)
        raycaster::draw_enemies(d, &self.enemies.list, &self.player, &self.zbuffer, &self.tex, self.render_scale, &self.fog_of_war);
        
        // HUD (FPS/HP/minimap)
        let fps_i = d.get_fps();
        let fps = fps_i as f32;
        // Draw HUD on the top-right to avoid overlapping the minimap
        let sw = d.get_screen_width();
        let hud_x = sw - 300;
        d.draw_text(&format!("FPS: {}", fps_i), hud_x, 10, 20, Color::LIGHTGRAY);
        // music status
        let music_status = if self.muted { "Muted".to_string() } else { format!("On {}%", (self.music_volume * 100.0) as i32) };
        d.draw_text(&format!("Music: {} (M to toggle)", music_status), hud_x, 30, 20, Color::LIGHTGRAY);
        d.draw_text(&format!("Render scale: {} (Z/X to change)", self.render_scale), hud_x, 50, 20, Color::LIGHTGRAY);
        // Coin counter
        d.draw_text(&format!("Coins: {}", self.coins_collected), hud_x, 70, 20, Color::GOLD);
        
        // === MODIFICADO: HUD de juego con pistol view ===
        ui::draw_game_hud(d, &self.player, &self.tex);
        
        // Auto-adjust render scale más conservador para mantener calidad
        if self.scale_cooldown <= 0 {
            if fps < 58.0 && self.render_scale < 8 {
                self.render_scale += 1;
                self.scale_cooldown = 60;
            } else if fps > 62.0 && self.render_scale > 1 {
                self.render_scale -= 1;
                self.scale_cooldown = 120; // Más lento al bajar para mantener calidad
            }
        }
        ui::draw_minimap(d, &self.maze, &self.player, Some(&self.fog_of_war)); // Pasar fog of war al minimap
        
        // Draw pause overlay if paused
        if self.paused {
            let w = d.get_screen_width();
            let h = d.get_screen_height();
            d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, 150));
            d.draw_text("Paused", w/2 - 60, h/2 - 40, 40, Color::WHITE);
            d.draw_text("Press ESC to resume, Q to quit to menu", w/2 - 180, h/2 + 10, 20, Color::LIGHTGRAY);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Drop for GameScreen {
    fn drop(&mut self) {
        if let Some(m) = &self.music { audio::stop_music(m); }
    }
}
