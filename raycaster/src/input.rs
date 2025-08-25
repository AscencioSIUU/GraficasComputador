use raylib::prelude::*;
use crate::player::Player;
use crate::maze::{Maze, is_wall, block_size};

pub fn process(rl: &RaylibHandle, player: &mut Player, maze: &Maze) {
    const ROT: f32 = std::f32::consts::PI / 180.0 * 2.0;
    const SPEED: f32 = 2.5;

    if rl.is_key_down(KeyboardKey::KEY_LEFT)  { player.a -= ROT; }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) { player.a += ROT; }

    let mut forward: f32 = 0.0; // <-- add the type
    if rl.is_key_down(KeyboardKey::KEY_UP)   { forward += 1.0; }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) { forward -= 1.0; }

    if forward.abs() > 0.0 {
        let dx = player.a.cos() * SPEED * forward;
        let dy = player.a.sin() * SPEED * forward;
        let next_x = player.pos.x + dx;
        let next_y = player.pos.y + dy;

        let b = block_size() as f32;
        let radius = 10.0;

        let corners = [
            (next_x - radius, next_y - radius),
            (next_x + radius, next_y - radius),
            (next_x - radius, next_y + radius),
            (next_x + radius, next_y + radius),
        ];

        let mut hit = false;
        for (x, y) in corners {
            let ci = (x / b) as isize;
            let cj = (y / b) as isize;
            if is_wall(maze, ci, cj) { hit = true; break; }
        }

        if !hit {
            player.pos.x = next_x;
            player.pos.y = next_y;
        }
    }
}
