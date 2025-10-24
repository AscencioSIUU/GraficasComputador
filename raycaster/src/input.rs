use raylib::prelude::*;
use crate::player::Player;
use crate::maze::{Maze, is_wall, block_size};

pub fn process(rl: &RaylibHandle, player: &mut Player, maze: &Maze) {
    let move_speed = 3.0;
    let rot_speed = 0.05;

    // W/S - Adelante/Atrás
    if rl.is_key_down(KeyboardKey::KEY_W) {
        let new_x = player.pos.x + player.a.cos() * move_speed;
        let new_y = player.pos.y + player.a.sin() * move_speed;
        if can_move(new_x, new_y, maze) {
            player.pos.x = new_x;
            player.pos.y = new_y;
        }
    }
    if rl.is_key_down(KeyboardKey::KEY_S) {
        let new_x = player.pos.x - player.a.cos() * move_speed;
        let new_y = player.pos.y - player.a.sin() * move_speed;
        if can_move(new_x, new_y, maze) {
            player.pos.x = new_x;
            player.pos.y = new_y;
        }
    }

    // A/D - Movimiento lateral (strafe izquierda/derecha)
    if rl.is_key_down(KeyboardKey::KEY_A) {
        // Mover perpendicular a la dirección (90 grados a la izquierda)
        let strafe_angle = player.a - std::f32::consts::PI / 2.0;
        let new_x = player.pos.x + strafe_angle.cos() * move_speed;
        let new_y = player.pos.y + strafe_angle.sin() * move_speed;
        if can_move(new_x, new_y, maze) {
            player.pos.x = new_x;
            player.pos.y = new_y;
        }
    }
    if rl.is_key_down(KeyboardKey::KEY_D) {
        // Mover perpendicular a la dirección (90 grados a la derecha)
        let strafe_angle = player.a + std::f32::consts::PI / 2.0;
        let new_x = player.pos.x + strafe_angle.cos() * move_speed;
        let new_y = player.pos.y + strafe_angle.sin() * move_speed;
        if can_move(new_x, new_y, maze) {
            player.pos.x = new_x;
            player.pos.y = new_y;
        }
    }

    // Flechas izquierda/derecha - Rotar cámara
    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a -= rot_speed;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a += rot_speed;
    }

    // Normalizar ángulo entre -PI y PI
    while player.a > std::f32::consts::PI {
        player.a -= 2.0 * std::f32::consts::PI;
    }
    while player.a < -std::f32::consts::PI {
        player.a += 2.0 * std::f32::consts::PI;
    }
}

fn can_move(x: f32, y: f32, maze: &Maze) -> bool {
    let b = block_size() as f32;
    let radius = 10.0;

    let corners = [
        (x - radius, y - radius),
        (x + radius, y - radius),
        (x - radius, y + radius),
        (x + radius, y + radius),
    ];

    let mut hit = false;
    for (x, y) in corners {
        let ci = (x / b) as isize;
        let cj = (y / b) as isize;
        if is_wall(maze, ci, cj) { hit = true; break; }
    }

    !hit
}
