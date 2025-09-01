use crate::{
    maze::Maze,
    player::Player,
    textures::{TextureManager, wall_key_from_char},
};
use raylib::prelude::*;

/// One wall hit returned by DDA
struct Hit {
    distance: f32, // perpendicular distance in *cell units*
    cell_x: i32,
    cell_y: i32,
    side: i32,     // 0 = hit on X-side (vertical wall), 1 = hit on Y-side (horizontal wall)
    impact: char,  // map char
    wall_x: f32,   // 0..1 coordinate along the wall for texturing
}

pub fn draw_world(
    d: &mut RaylibDrawHandle,
    maze: &Maze,
    player: &Player,
    zbuffer: &mut [f32],
    tex: &TextureManager,
) {
    let screen_w = d.get_screen_width().max(1) as usize;
    let screen_h = d.get_screen_height().max(1) as usize;

    d.clear_background(Color::BLACK);

    // --- Projection setup (use screen WIDTH because FOV is horizontal) ---
    let fov = player.fov;
    let half_w = screen_w as f32 * 0.5;
    let half_h = screen_h as f32 * 0.5;
    let proj_plane = half_w / (fov * 0.5).tan();

    for x in 0..screen_w {
        // Map column x -> camera space (-1 .. +1)
        let camera_x = (x as f32 - half_w) / half_w;
        let ray_angle = player.a + camera_x * (fov * 0.5);

        // Shared ray direction (unit) for ceiling/floor
        let ray_dir_x = ray_angle.cos();
        let ray_dir_y = ray_angle.sin();

        let hit = cast_ray_dda(maze, player, ray_angle);
        zbuffer[x] = hit.distance;

        // Projected wall slice height in pixels (cells have height 1.0)
        let line_h_f = (1.0 * proj_plane) / hit.distance.max(1e-4);
        let mut line_h = line_h_f as i32;

        if line_h > screen_h as i32 { line_h = screen_h as i32; }
        let mut draw_start = (-line_h / 2) + (screen_h as i32 / 2);
        if draw_start < 0 { draw_start = 0; }
        let mut draw_end = (line_h / 2) + (screen_h as i32 / 2);
        if draw_end >= screen_h as i32 { draw_end = screen_h as i32 - 1; }

        // === CEILING (tiled like floor; same FOV & distance) ===
        let ceiling_key = "ceiling"; // use "floor" here if you don't have a ceiling texture
        let (ctw, cth) = tex.size_of(ceiling_key);
        let cell = crate::maze::block_size() as f32;

        // From the top down to start of the wall slice
        for y in 0..draw_start.max(0) as usize {
            // distance from screen center in pixels (above horizon)
            let p = (half_h - y as f32).max(1.0);
            // world row distance using the SAME projection plane
            let row_dist_world = (proj_plane * cell) / p;

            // mirror of floor: move backwards along the ray
            let world_x = player.pos.x - ray_dir_x * row_dist_world;
            let world_y = player.pos.y - ray_dir_y * row_dist_world;

            // local fraction in cell [0,1)
            let fx = (world_x.rem_euclid(cell)) / cell; // see Rust docs: f32::rem_euclid
            let fy = (world_y.rem_euclid(cell)) / cell;

            // texture coords
            let txu = (fx * (ctw - 1) as f32) as u32;
            let tyu = (fy * (cth - 1) as f32) as u32;

            let mut color = tex.get_pixel_color(ceiling_key, txu, tyu);
            // fog by distance (in cells)
            let dist_in_cells = row_dist_world / cell;
            fog_with_distance(&mut color, dist_in_cells, 0.12);
            d.draw_pixel(x as i32, y as i32, color);
        }

        // === WALL slice (textured) ===
        let wall_key = wall_key_from_char(hit.impact);
        let (tw, th) = tex.size_of(wall_key);
        let tx_u32 = ((hit.wall_x * (tw as f32 - 1.0)) as u32).min(tw - 1);

        for y in draw_start..=draw_end {
            let v = (y - draw_start) as f32 / (draw_end - draw_start + 1) as f32;
            let ty = ((v * (th as f32 - 1.0)) as u32).min(th - 1);

            let mut color = tex.get_pixel_color(wall_key, tx_u32, ty);
            if hit.side == 1 { darken(&mut color, 0.85); }
            fog_with_distance(&mut color, hit.distance, 0.015);
            d.draw_pixel(x as i32, y, color);
        }

        // === FLOOR (tiled per cell, same FOV & distance) ===
        let (ftw, fth) = tex.size_of("floor");

        for y in (draw_end + 1) as usize..screen_h {
            // distance from screen center in pixels (below horizon)
            let p = (y as f32 - half_h).max(1.0);
            // world row distance
            let row_dist_world = (proj_plane * cell) / p;

            // move forward along the ray
            let world_x = player.pos.x + ray_dir_x * row_dist_world;
            let world_y = player.pos.y + ray_dir_y * row_dist_world;

            let fx = (world_x.rem_euclid(cell)) / cell;
            let fy = (world_y.rem_euclid(cell)) / cell;

            let tx = (fx * (ftw - 1) as f32) as u32;
            let ty = (fy * (fth - 1) as f32) as u32;

            let mut color = tex.get_pixel_color("floor", tx, ty);
            let dist_in_cells = row_dist_world / cell;
            fog_with_distance(&mut color, dist_in_cells, 0.12);
            d.draw_pixel(x as i32, y as i32, color);
        }
    }
}

/// DDA ray casting on a grid (cell units). Returns wall texture X coordinate too.
fn cast_ray_dda(maze: &Maze, player: &Player, angle: f32) -> Hit {
    let b = crate::maze::block_size() as f32;

    // Position in cell space
    let pos_x = player.pos.x / b;
    let pos_y = player.pos.y / b;

    // Ray direction in cell space (unit)
    let ray_dir_x = angle.cos();
    let ray_dir_y = angle.sin();

    let mut map_x = pos_x.floor() as i32;
    let mut map_y = pos_y.floor() as i32;

    let delta_dist_x = if ray_dir_x.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_x).abs() };
    let delta_dist_y = if ray_dir_y.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_y).abs() };

    let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
        let sd = (pos_x - map_x as f32) * delta_dist_x;
        (-1, sd)
    } else {
        let sd = (map_x as f32 + 1.0 - pos_x) * delta_dist_x;
        (1, sd)
    };

    let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
        let sd = (pos_y - map_y as f32) * delta_dist_y;
        (-1, sd)
    } else {
        let sd = (map_y as f32 + 1.0 - pos_y) * delta_dist_y;
        (1, sd)
    };

    // DDA loop
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

        // out of bounds -> treat as solid
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

    // Perpendicular distance in *cell units* (no fisheye)
    let perp_dist = if side == 0 {
        (map_x as f32 - pos_x + (1 - step_x) as f32 * 0.5) / (ray_dir_x + 1e-6)
    } else {
        (map_y as f32 - pos_y + (1 - step_y) as f32 * 0.5) / (ray_dir_y + 1e-6)
    }
    .abs()
    .max(0.0001);

    // Exact impact point in world units (for wall_x)
    let hit_x_world = player.pos.x + ray_dir_x * perp_dist * b;
    let hit_y_world = player.pos.y + ray_dir_y * perp_dist * b;

    // wall_x in 0..1 along the wall
    let mut wall_x = if side == 0 {
        // vertical wall: use y fraction in the cell
        (hit_y_world / b) - (hit_y_world / b).floor()
    } else {
        // horizontal wall: use x fraction in the cell
        (hit_x_world / b) - (hit_x_world / b).floor()
    };

    // Flip to avoid mirrored textures depending on ray direction
    if (side == 0 && ray_dir_x > 0.0) || (side == 1 && ray_dir_y < 0.0) {
        wall_x = 1.0 - wall_x;
    }

    Hit {
        distance: perp_dist,
        cell_x: map_x,
        cell_y: map_y,
        side,
        impact,
        wall_x,
    }
}

// --- Small helpers for shading/fog ---

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
    // simple exponential fog
    let fog = (1.0 - (-density * dist).exp()).clamp(0.0, 1.0);
    apply_fog(c, fog);
}
