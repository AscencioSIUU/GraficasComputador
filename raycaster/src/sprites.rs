use raylib::prelude::*;
use crate::{framebuffer::Framebuffer, player::Player, textures::TextureManager};

#[derive(Clone, Copy, Default)]
pub struct Enemy { pub pos: Vector2, pub texture_key: char, pub frame: u32 }

#[derive(Default)]
pub struct EnemySystem { list: Vec<Enemy> }
impl EnemySystem {
    pub fn update(&mut self) {
        for e in &mut self.list {
            e.frame = (e.frame + 1) % 4; // p. ej. 4 frames
        }
    }
    pub fn draw(&self, fb: &mut Framebuffer, player: &Player, zbuffer: &[f32], tex: &TextureManager) {
        for e in &self.list {
            draw_sprite(fb, player, e, zbuffer, tex);
        }
    }
}

fn draw_sprite(
    fb: &mut Framebuffer,
    player: &Player,
    enemy: &Enemy,
    zbuffer: &[f32],
    tex: &TextureManager
) {
    // Proyección como en tu función; además compara con zbuffer[x] para oclusión
    // Reusa tu lógica de ángulo/diferencia/size → sampleo por (tx, ty) y frame-column
    // …
}
