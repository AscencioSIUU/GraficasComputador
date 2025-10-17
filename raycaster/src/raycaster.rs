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
    render_scale: usize,
) {
    let screen_w = d.get_screen_width().max(1) as usize;
    let screen_h = d.get_screen_height().max(1) as usize;

    d.clear_background(Color::BLACK);

    // --- Projection setup (use screen WIDTH because FOV is horizontal) ---
    let fov = player.fov;
    let half_w = screen_w as f32 * 0.5;
    let half_h = screen_h as f32 * 0.5;
    let proj_plane = half_w / (fov * 0.5).tan();

    let step_x = render_scale.max(1);
    let step_y = render_scale.max(1);
    let mut sx = 0usize;
    let mut col_index = 0usize;
    while sx < screen_w {
        // Map column sx -> camera space (-1 .. +1)
        let camera_x = (sx as f32 - half_w) / half_w;
        let ray_angle = player.a + camera_x * (fov * 0.5);

        // Shared ray direction (unit) for ceiling/floor
    let ray_dir_x = ray_angle.cos();
    let ray_dir_y = ray_angle.sin();

        let hit = cast_ray_dda(maze, player, ray_angle);
    // write into zbuffer indexed by scaled columns
    if col_index < zbuffer.len() { zbuffer[col_index] = hit.distance; }

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
        let mut sy = 0usize;
        while sy < draw_start.max(0) as usize {
            // distance from screen center in pixels (above horizon)
            let p = (half_h - sy as f32).max(1.0);
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
            // draw scaled pixel block
            if render_scale <= 1 {
                d.draw_pixel(sx as i32, sy as i32, color);
            } else {
                d.draw_rectangle(sx as i32, sy as i32, step_x as i32, step_y as i32, color);
            }
            sy += step_y;
        }

        // === WALL slice (textured) ===
        let wall_key = wall_key_from_char(hit.impact);
        let (tw, th) = tex.size_of(wall_key);
        let tx_u32 = ((hit.wall_x * (tw as f32 - 1.0)) as u32).min(tw - 1);

        let mut sy2 = draw_start as usize;
        while sy2 <= draw_end as usize {
            let v = (sy2 - draw_start as usize) as f32 / (draw_end - draw_start + 1) as f32;
            let ty = ((v * (th as f32 - 1.0)) as u32).min(th - 1);

            let mut color = tex.get_pixel_color(wall_key, tx_u32, ty);
            if hit.side == 1 { darken(&mut color, 0.85); }
            fog_with_distance(&mut color, hit.distance, 0.015);
            if render_scale <= 1 {
                d.draw_pixel(sx as i32, sy2 as i32, color);
            } else {
                d.draw_rectangle(sx as i32, sy2 as i32, step_x as i32, step_y as i32, color);
            }
            sy2 += step_y;
        }

        // === FLOOR (tiled per cell, same FOV & distance) ===
        let (ftw, fth) = tex.size_of("floor");

        let mut sy3 = (draw_end + 1) as usize;
        while sy3 < screen_h {
            // distance from screen center in pixels (below horizon)
            let p = (sy3 as f32 - half_h).max(1.0);
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
            if render_scale <= 1 {
                d.draw_pixel(sx as i32, sy3 as i32, color);
            } else {
                d.draw_rectangle(sx as i32, sy3 as i32, step_x as i32, step_y as i32, color);
            }
            sy3 += step_y;
        }
        sx += step_x;
        col_index += 1;
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

/// Render coins as billboard sprites in the 3D world
pub fn draw_coins(
    d: &mut RaylibDrawHandle,
    maze: &Maze,
    player: &Player,
    zbuffer: &[f32],
    tex: &TextureManager,
    render_scale: usize,
) {
    let screen_w = d.get_screen_width().max(1) as usize;
    let screen_h = d.get_screen_height().max(1) as usize;
    let block_size = crate::maze::block_size() as f32;
    
    // Collect all coin positions
    let mut coins = Vec::new();
    for (y, row) in maze.iter().enumerate() {
        for (x, &ch) in row.iter().enumerate() {
            if ch == 'X' || ch == 'x' {
                // Center of the tile
                let world_x = (x as f32 + 0.5) * block_size;
                let world_y = (y as f32 + 0.5) * block_size;
                coins.push((world_x, world_y));
            }
        }
    }
    
    // Sort coins by distance (farthest first for proper occlusion)
    coins.sort_by(|a, b| {
        let da = ((a.0 - player.pos.x).powi(2) + (a.1 - player.pos.y).powi(2)).sqrt();
        let db = ((b.0 - player.pos.x).powi(2) + (b.1 - player.pos.y).powi(2)).sqrt();
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    let fov = player.fov;
    let half_w = screen_w as f32 * 0.5;
    let half_h = screen_h as f32 * 0.5;
    let proj_plane = half_w / (fov * 0.5).tan();
    
    // Get coin texture
    let coin_key = "coin";
    let (tex_w, tex_h) = tex.size_of(coin_key);
    
    for (coin_x, coin_y) in coins {
        // Vector from player to coin
        let dx = coin_x - player.pos.x;
        let dy = coin_y - player.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance < 1.0 { continue; } // Too close
        
        // Angle to coin relative to player direction
        let angle_to_coin = dy.atan2(dx);
        let mut angle_diff = angle_to_coin - player.a;
        
        // Normalize angle to -PI..PI
        while angle_diff > std::f32::consts::PI { angle_diff -= 2.0 * std::f32::consts::PI; }
        while angle_diff < -std::f32::consts::PI { angle_diff += 2.0 * std::f32::consts::PI; }
        
        // Check if coin is within FOV
        if angle_diff.abs() > fov * 0.5 + 0.2 { continue; }
        
        // Project to screen x
        let screen_x = ((angle_diff / (fov * 0.5)) * half_w + half_w) as i32;
        
        // Sprite height based on distance (coins are smaller than full block height)
        let sprite_height = (0.5 * proj_plane / distance.max(1e-4)) as i32;
        let sprite_width = sprite_height; // Square sprite
        
        let draw_start_y = (half_h as i32 - sprite_height / 2).max(0);
        let draw_end_y = (half_h as i32 + sprite_height / 2).min(screen_h as i32 - 1);
        let draw_start_x = (screen_x - sprite_width / 2).max(0);
        let draw_end_x = (screen_x + sprite_width / 2).min(screen_w as i32 - 1);
        
        // Draw sprite column by column with zbuffer check
        for sx in (draw_start_x..=draw_end_x).step_by(render_scale.max(1)) {
            let col_index = (sx as usize) / render_scale.max(1);
            
            // Check zbuffer for occlusion
            if col_index < zbuffer.len() && distance > zbuffer[col_index] {
                continue; // Coin is behind a wall
            }
            
            // Texture x coordinate
            let tx = ((sx - (screen_x - sprite_width / 2)) as f32 / sprite_width as f32 * tex_w as f32) as usize;
            let tx = tx.min(tex_w - 1);
            
            for sy in (draw_start_y..=draw_end_y).step_by(render_scale.max(1)) {
                // Texture y coordinate
                let ty = ((sy - draw_start_y) as f32 / sprite_height as f32 * tex_h as f32) as usize;
                let ty = ty.min(tex_h - 1);
                
                // Sample texture
                let mut color = tex.sample_at(coin_key, tx, ty);
                
                // Skip transparent pixels (assuming black or very dark = transparent)
                if color.r < 10 && color.g < 10 && color.b < 10 {
                    continue;
                }
                
                // Apply fog
                fog_with_distance(&mut color, distance / block_size, 0.15);
                
                // Draw pixel or block depending on render_scale
                if render_scale > 1 {
                    d.draw_rectangle(sx, sy, render_scale as i32, render_scale as i32, color);
                } else {
                    d.draw_pixel(sx, sy, color);
                }
            }
        }
    }
}
