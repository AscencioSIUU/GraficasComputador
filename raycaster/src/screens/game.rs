use raylib::prelude::*;
use super::Screen;
use crate::{
    player::Player,
    maze::{Maze, load_maze},
    ui, input,
    textures::TextureManager,
    raycaster,
};

pub struct GameScreen {
    maze: Maze,
    player: Player,
    level_path: String,
    tex: TextureManager,
    zbuffer: Vec<f32>,
}

impl GameScreen {
    pub fn new(level_path: String) -> Self {
        let maze = load_maze(&level_path);
        let player = Player::new(Vector2::new(96.0, 96.0));
        let tex = TextureManager::from_assets().expect("load textures");
        let zbuffer = vec![f32::INFINITY; 2048]; // will be resized first frame
        Self { maze, player, level_path, tex, zbuffer }
    }
}

impl Screen for GameScreen {
    fn update(&mut self, rl: &RaylibHandle) {
        input::process(rl, &mut self.player, &self.maze);
    }

    fn draw_raylib(&mut self, d: &mut RaylibDrawHandle) {
        // make sure zbuffer matches current screen width
        let w = d.get_screen_width().max(1) as usize;
        if self.zbuffer.len() != w { self.zbuffer.resize(w, f32::INFINITY); }

        // 3D world
        raycaster::draw_world(d, &self.maze, &self.player, &mut self.zbuffer, &self.tex);

        // HUD (FPS/HP/minimap)
        let fps = d.get_fps();
        d.draw_text(&format!("FPS: {}", fps), 10, 10, 20, Color::LIGHTGRAY);
        ui::draw_minimap(d, &self.maze, &self.player);
    }
}
