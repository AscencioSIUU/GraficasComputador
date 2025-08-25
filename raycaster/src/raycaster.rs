use crate::{maze::Maze, player::Player, textures::{TextureManager, wall_key_from_char}};
use raylib::prelude::*;

/// A single wall hit
struct Hit {
    distance: f32, // perpendicular distance (no fisheye)
    cell_x: i32,
    cell_y: i32,
    side: i32,     // 0 = hit on X-side, 1 = hit on Y-side
    impact: char,  // map char we hit
    tx: u32,       // texture x (0..W-1)
}

/// Draw a full frame: ceiling, walls (textured), floor.
pub fn draw_world(
    d: &mut RaylibDrawHandle,
    maze: &Maze,
    player: &Player,
    zbuffer: &mut [f32],
    tex: &TextureManager,
) {
    let screen_w = d.get_screen_width().max(1) as usize;
    let screen_h = d.get_screen_height().max(1) as usize;

    // Draw background (we overwrite everything anyway)
    d.clear_background(Color::BLACK);

    let fov = player.fov; // ~ PI/2
    let (half_w, half_h) = (screen_w as f32 * 0.5, screen_h as f32 * 0.5);

    for x in 0..screen_w {
        // Convert screen column to a ray angle around player angle
        let camera_x = (x as f32 - half_w) / half_w;       // -1..+1
        let ray_angle = player.a + camera_x * (fov * 0.5); // spread by FOV

        let hit = cast_ray_dda(maze, player, ray_angle);

        // keep distance for sprite occlusion
        zbuffer[x] = hit.distance;

        // Projected wall height (simple)
        let line_h = (screen_h as f32 / (hit.distance + 0.0001)).min(screen_h as f32) as i32;
        let mut draw_start = (-line_h / 2) + (screen_h as i32 / 2);
        if draw_start < 0 { draw_start = 0; }
        let mut draw_end = (line_h / 2) + (screen_h as i32 / 2);
        if draw_end >= screen_h as i32 { draw_end = screen_h as i32 - 1; }

        // === Ceiling ===
        // quick-tile: repeat texture by x and screen-y; looks OK for a raycaster
        let (_, tex_h) = tex.size_of("ceiling");
        for y in 0..draw_start.max(0) {
            let ty = ((y as f32 / draw_start.max(1) as f32) * (tex_h - 1) as f32) as u32;
            let tx = (x as u32) & 127;
            let mut color = Color::new(120, 120, 120, 255);
            apply_fog(&mut color, 0.4);
            d.draw_pixel(x as i32, y as i32, color);
        }

        // === Wall slice ===
        let wall_key = wall_key_from_char(hit.impact);
        let (tw, th) = tex.size_of(wall_key);
        for y in draw_start..=draw_end {
            let v = (y - draw_start) as f32 / (draw_end - draw_start + 1) as f32; // 0..1
            let tx = hit.tx.min(tw - 1);
            let ty = (v * (th - 1) as f32) as u32;
            let mut color = tex.get_pixel_color(wall_key, tx, ty);

            // Simple shading: darken if we hit a Y-side, also fog with distance
            if hit.side == 1 { darken(&mut color, 0.85); }
            fog_with_distance(&mut color, hit.distance, 0.015);
            d.draw_pixel(x as i32, y, color);
        }

        // === Floor ===
        // quick-tile by screen coords (fast & looks fine in retro raycasters)
        let (_, fth) = tex.size_of("floor");
        for y in (draw_end + 1) as usize..screen_h {
            let ty = ((((y - draw_end as usize) as f32) / (screen_h - draw_end as usize) as f32) * (fth - 1) as f32) as u32;
            let tx = (x as u32) & 127;
            let mut color = tex.get_pixel_color("floor", tx, ty);
            // increase fog with vertical distance visually
            let fake_dist = 0.5 + 1.5 * ((y as f32 - half_h) / half_h).abs();
            fog_with_distance(&mut color, fake_dist, 0.25);
            d.draw_pixel(x as i32, y as i32, color);
        }
    }
}

/// DDA ray casting on grid (cell units). Returns wall texture X coordinate as well.
fn cast_ray_dda(maze: &Maze, player: &Player, angle: f32) -> Hit {
    let b = crate::maze::block_size() as f32;

    // position in cell space
    let mut pos_x = player.pos.x / b;
    let mut pos_y = player.pos.y / b;

    // ray direction in cell space
    let ray_dir_x = angle.cos();
    let ray_dir_y = angle.sin();

    let mut map_x = pos_x.floor() as i32;
    let mut map_y = pos_y.floor() as i32;

    let delta_dist_x = if ray_dir_x.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_x).abs() };
    let delta_dist_y = if ray_dir_y.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_y).abs() };

    let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
        let s = -1;
        let sd = (pos_x - map_x as f32) * delta_dist_x;
        (s, sd)
    } else {
        let s = 1;
        let sd = (map_x as f32 + 1.0 - pos_x) * delta_dist_x;
        (s, sd)
    };
    let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
        let s = -1;
        let sd = (pos_y - map_y as f32) * delta_dist_y;
        (s, sd)
    } else {
        let s = 1;
        let sd = (map_y as f32 + 1.0 - pos_y) * delta_dist_y;
        (s, sd)
    };

    // DDA
    let mut side = 0; // 0 x-side, 1 y-side
    let mut impact = ' ';
    let (h, w) = (maze.len() as i32, maze.first().map(|r| r.len()).unwrap_or(0) as i32);

    loop {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = 1;
        }
        // bounds
        if map_x < 0 || map_y < 0 || map_y >= h || map_x >= w {
            impact = '#';
            break;
        }
        let ch = maze[map_y as usize][map_x as usize];
        if ch != ' ' {
            impact = ch;
            break;
        }
    }

    // perpendicular wall distance (avoid fisheye)
    let perp_dist = if side == 0 {
        (map_x as f32 - pos_x + (1 - step_x) as f32 * 0.5) / (ray_dir_x + 1e-6)
    } else {
        (map_y as f32 - pos_y + (1 - step_y) as f32 * 0.5) / (ray_dir_y + 1e-6)
    }.abs();

    // find exact impact point along the wall to compute texture X
    let hit_x_world = player.pos.x + ray_dir_x * perp_dist * b;
    let hit_y_world = player.pos.y + ray_dir_y * perp_dist * b;

    // Which side and offset inside the cell:
    let mut wall_x = if side == 0 {
        // hit vertical wall, so use y within the cell
        (hit_y_world / b) - (hit_y_world / b).floor()
    } else {
        // hit horizontal wall, so use x within the cell
        (hit_x_world / b) - (hit_x_world / b).floor()
    };
    // flip to make textures not mirrored depending on ray direction
    if (side == 0 && ray_dir_x > 0.0) || (side == 1 && ray_dir_y < 0.0) {
        wall_x = 1.0 - wall_x;
    }
    // Texture X coordinate (assuming 128px textures)
    let tx = (wall_x * 127.0) as u32;

    Hit {
        distance: perp_dist.max(0.0001),
        cell_x: map_x,
        cell_y: map_y,
        side,
        impact,
        tx,
    }
}

// === Small helpers for shading/fog ===
fn darken(c: &mut Color, k: f32) {
    c.r = ((c.r as f32) * k) as u8;
    c.g = ((c.g as f32) * k) as u8;
    c.b = ((c.b as f32) * k) as u8;
}

fn apply_fog(c: &mut Color, fog: f32) {
    let k = (1.0 - fog).clamp(0.0, 1.0);
    darken(c, k);
}

fn fog_with_distance(c: &mut Color, dist: f32, density: f32) {
    let fog = (1.0 - (-density * dist).exp()).clamp(0.0, 1.0);
    apply_fog(c, fog);
}
