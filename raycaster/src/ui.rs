use raylib::prelude::*;
use crate::{maze::{Maze, block_size}, player::Player}; // ← agrega block_size

pub fn draw_minimap(d: &mut RaylibDrawHandle, maze: &Maze, player: &Player) {
    let scale = 4;
    let origin_x = 10;
    let origin_y = 40;
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            let wall = matches!(c, '#' | 'A' | 'B' | 'C');
            let color = if wall { Color::DARKBLUE } else { Color::DARKGRAY };
            d.draw_rectangle(origin_x + i as i32 * scale,
                             origin_y + j as i32 * scale,
                             scale, scale, color);
        }
    }
    // jugador
    let bs = block_size() as i32;
    d.draw_circle(
        origin_x + (player.pos.x as i32 * scale / bs),
        origin_y + (player.pos.y as i32 * scale / bs),
        2.0,
        Color::YELLOW
    );
}
