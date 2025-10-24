use crate::{
    maze::Maze,
    player::Player,
    textures::TextureManager,
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

    let fov = player.fov;
    let half_w = screen_w as f32 * 0.5;
    let half_h = screen_h as f32 * 0.5;
    let proj_plane = half_w / (fov * 0.5).tan();

    let step_x = render_scale.max(1);
    let step_y = render_scale.max(1);
    let mut sx = 0usize;
    let mut col_index = 0usize;
    
    while sx < screen_w {
        let camera_x = (sx as f32 - half_w) / half_w;
        let ray_angle = player.a + camera_x * (fov * 0.5);

        let ray_dir_x = ray_angle.cos();
        let ray_dir_y = ray_angle.sin();

        let hit = cast_ray_dda(maze, player, ray_angle);
        if col_index < zbuffer.len() { zbuffer[col_index] = hit.distance; }

        let line_h_f = (1.0 * proj_plane) / hit.distance.max(1e-4);
        let mut line_h = line_h_f as i32;

        if line_h > screen_h as i32 { line_h = screen_h as i32; }
        let mut draw_start = (-line_h / 2) + (screen_h as i32 / 2);
        if draw_start < 0 { draw_start = 0; }
        let mut draw_end = (line_h / 2) + (screen_h as i32 / 2);
        if draw_end >= screen_h as i32 { draw_end = screen_h as i32 - 1; }

        // CEILING
        let ceiling_key = "ceiling";
        let (ctw, cth) = tex.size_of(ceiling_key);
        let cell = crate::maze::block_size() as f32;

        let mut sy = 0usize;
        while sy < draw_start.max(0) as usize {
            let p = (half_h - sy as f32).max(1.0);
            let row_dist_world = (proj_plane * cell) / p;
            let world_x = player.pos.x - ray_dir_x * row_dist_world;
            let world_y = player.pos.y - ray_dir_y * row_dist_world;
            let fx = (world_x.rem_euclid(cell)) / cell;
            let fy = (world_y.rem_euclid(cell)) / cell;
            let txu = (fx * (ctw - 1) as f32) as u32;
            let tyu = (fy * (cth - 1) as f32) as u32;
            let mut color = tex.get_pixel_color(ceiling_key, txu, tyu);
            let dist_in_cells = row_dist_world / cell;
            fog_with_distance(&mut color, dist_in_cells, 0.12);
            if render_scale <= 1 {
                d.draw_pixel(sx as i32, sy as i32, color);
            } else {
                d.draw_rectangle(sx as i32, sy as i32, step_x as i32, step_y as i32, color);
            }
            sy += step_y;
        }

        // WALL
        let wall_key = crate::textures::wall_key_from_char_at(hit.impact, hit.cell_x, hit.cell_y);
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

        // FLOOR
        let (ftw, fth) = tex.size_of("floor");
        let mut sy3 = (draw_end + 1) as usize;
        while sy3 < screen_h {
            let p = (sy3 as f32 - half_h).max(1.0);
            let row_dist_world = (proj_plane * cell) / p;
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

fn cast_ray_dda(maze: &Maze, player: &Player, angle: f32) -> Hit {
    let b = crate::maze::block_size() as f32;
    let pos_x = player.pos.x / b;
    let pos_y = player.pos.y / b;
    let ray_dir_x = angle.cos();
    let ray_dir_y = angle.sin();
    let mut map_x = pos_x.floor() as i32;
    let mut map_y = pos_y.floor() as i32;
    let delta_dist_x = if ray_dir_x.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_x).abs() };
    let delta_dist_y = if ray_dir_y.abs() < 1e-6 { 1e30 } else { (1.0 / ray_dir_y).abs() };
    let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
        (-1, (pos_x - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as f32 + 1.0 - pos_x) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
        (-1, (pos_y - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as f32 + 1.0 - pos_y) * delta_dist_y)
    };
    let mut side = 0;
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
        if map_x < 0 || map_y < 0 || map_y >= h || map_x >= w {
            impact = '#';
            break;
        }
        let ch = maze[map_y as usize][map_x as usize];
        if ch != ' ' && ch != 'X' && ch != 'x' && ch != 'P' && ch != 'p' {
            impact = ch;
            break;
        }
    }
    let perp_dist = if side == 0 {
        (map_x as f32 - pos_x + (1 - step_x) as f32 * 0.5) / (ray_dir_x + 1e-6)
    } else {
        (map_y as f32 - pos_y + (1 - step_y) as f32 * 0.5) / (ray_dir_y + 1e-6)
    }.abs().max(0.0001);
    let hit_x_world = player.pos.x + ray_dir_x * perp_dist * b;
    let hit_y_world = player.pos.y + ray_dir_y * perp_dist * b;
    let mut wall_x = if side == 0 {
        (hit_y_world / b) - (hit_y_world / b).floor()
    } else {
        (hit_x_world / b) - (hit_x_world / b).floor()
    };
    if (side == 0 && ray_dir_x > 0.0) || (side == 1 && ray_dir_y < 0.0) {
        wall_x = 1.0 - wall_x;
    }
    Hit { distance: perp_dist, cell_x: map_x, cell_y: map_y, side, impact, wall_x }
}

fn darken(c: &mut Color, k: f32) {
    c.r = ((c.r as f32) * k) as u8;
    c.g = ((c.g as f32) * k) as u8;
    c.b = ((c.b as f32) * k) as u8;
}

fn apply_fog(c: &mut Color, fog: f32) {
    let k = (1.0 - fog).clamp(0.0, 1.0);
    darken(c, k);
}

pub fn fog_with_distance(c: &mut Color, dist: f32, density: f32) {
    let fog = (1.0 - (-density * dist).exp()).clamp(0.0, 1.0);
    apply_fog(c, fog);
}

/// Render coins as billboard sprites in the 3D world (cambios mínimos)
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
    
    // Recolectar monedas
    let mut coins = Vec::new();
    for (y, row) in maze.iter().enumerate() {
        for (x, &ch) in row.iter().enumerate() {
            if ch == 'X' || ch == 'x' {
                let world_x = (x as f32 + 0.5) * block_size;
                let world_y = (y as f32 + 0.5) * block_size;
                coins.push((world_x, world_y));
            }
        }
    }
    
    // Orden: lejos -> cerca (lo dejo igual que tenías)
    coins.sort_by(|a, b| {
        let da = ((a.0 - player.pos.x).powi(2) + (a.1 - player.pos.y).powi(2)).sqrt();
        let db = ((b.0 - player.pos.x).powi(2) + (b.1 - player.pos.y).powi(2)).sqrt();
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    let fov = player.fov;
    let half_w = screen_w as f32 * 0.5;
    let half_h = screen_h as f32 * 0.5;
    let proj_plane = half_w / (fov * 0.5).tan();
    
    // Textura
    let coin_key = "coin";
    let (tex_w_u32, tex_h_u32) = tex.size_of(coin_key);
    let (tex_w, tex_h) = (tex_w_u32 as usize, tex_h_u32 as usize);

    // === NUEVO: vector "forward" del jugador ===
    let fwd_x = player.a.cos();
    let fwd_y = player.a.sin();

    for (coin_x, coin_y) in coins {
        // Vector del jugador a la moneda
        let dx = coin_x - player.pos.x;
        let dy = coin_y - player.pos.y;

        // Ángulo relativo (igual que lo tenías)
        let angle_to_coin = dy.atan2(dx);
        let mut angle_diff = angle_to_coin - player.a;
        while angle_diff > std::f32::consts::PI { angle_diff -= 2.0 * std::f32::consts::PI; }
        while angle_diff < -std::f32::consts::PI { angle_diff += 2.0 * std::f32::consts::PI; }

        // Culling por FOV (mantengo tu margen)
        if angle_diff.abs() > fov * 0.6 { continue; }

        // === NUEVO: profundidad perpendicular (no distancia radial) ===
        let depth_world = dx * fwd_x + dy * fwd_y; // dot hacia adelante
        if depth_world <= 1e-4 { continue; }       // detrás de la cámara
        let depth_cells = depth_world / block_size;

        // Proyección X
        let screen_x = ((angle_diff / (fov * 0.5)) * half_w + half_w) as i32;

        // === CAMBIO: tamaño con profundidad perpendicular en celdas ===
        let sprite_height = (0.6 * proj_plane / depth_cells.max(1e-4)) as i32;
        if sprite_height < 1 { continue; }
        let sprite_width = sprite_height; // cuadrado
        
        // Posiciones de dibujo (igual que lo tenías)
        let draw_start_y = ((half_h as i32) - sprite_height / 2 + sprite_height / 6).max(0);
        let draw_end_y = (draw_start_y + sprite_height).min(screen_h as i32 - 1);
        let draw_start_x = (screen_x - sprite_width / 2).max(0);
        let draw_end_x = (screen_x + sprite_width / 2).min(screen_w as i32 - 1);
        if draw_end_x <= draw_start_x { continue; }
        
        // Dibujar con zbuffer
        let rs = render_scale.max(1);
        for sx in (draw_start_x..=draw_end_x).step_by(rs) {
            let col_index = (sx as usize) / rs;

            // === CAMBIO: comparar zbuffer contra profundidad perpendicular en celdas ===
            if col_index < zbuffer.len() && depth_cells > zbuffer[col_index] * 0.98 {
                continue; // detrás de una pared
            }
            
            // Texcoord X - ARREGLADO: agregar paréntesis correctos
            let tx = ((((sx - (screen_x - sprite_width / 2)) as f32 / sprite_width as f32)
                * (tex_w as f32 - 1.0)) as usize)
                .min(tex_w.saturating_sub(1));
            
            for sy in (draw_start_y..=draw_end_y).step_by(rs) {
                // Texcoord Y - ARREGLADO: agregar paréntesis correctos
                let ty = ((((sy - draw_start_y) as f32 / sprite_height as f32)
                    * (tex_h as f32 - 1.0)) as usize)
                    .min(tex_h.saturating_sub(1));
                
                // Sample
                let mut color = tex.sample_at(coin_key, tx, ty);

                // Alfa (lo dejo bajo para bordes suaves)
                if color.a < 10 { continue; }
                
                // === CAMBIO: fog usando profundidad en celdas - ARREGLADO: llamar directamente ===
                fog_with_distance(&mut color, depth_cells, 0.08);
                
                // Dibujo según render_scale
                if rs > 1 {
                    d.draw_rectangle(sx, sy, rs as i32, rs as i32, color);
                } else {
                    d.draw_pixel(sx, sy, color);
                }
            }
        }
    }
}

/// Render enemies as billboard sprites in the 3D world
pub fn draw_enemies(
    d: &mut RaylibDrawHandle,
    enemies: &[crate::sprites::Enemy],
    player: &Player,
    zbuffer: &[f32],
    tex: &TextureManager,
    render_scale: usize,
) {
    let screen_w = d.get_screen_width().max(1) as usize;
    let screen_h = d.get_screen_height().max(1) as usize;
    let block_size = crate::maze::block_size() as f32;
    
    // Sort enemies by distance (farthest first)
    let mut sorted_enemies: Vec<_> = enemies.iter()
        .filter(|e| e.alive)
        .map(|e| {
            let dx = e.pos.x - player.pos.x;
            let dy = e.pos.y - player.pos.y;
            let dist = (dx * dx + dy * dy).sqrt();
            (e, dist)
        })
        .collect();
    
    sorted_enemies.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    let fov = player.fov;
    let half_w = screen_w as f32 * 0.5;
    let half_h = screen_h as f32 * 0.5;
    let proj_plane = half_w / (fov * 0.5).tan();
    
    let enemy_key = "enemy";
    let (tex_w_u32, tex_h_u32) = tex.size_of(enemy_key);
    let (tex_w, tex_h) = (tex_w_u32 as usize, tex_h_u32 as usize);
    
    let fwd_x = player.a.cos();
    let fwd_y = player.a.sin();
    
    for (enemy, _) in sorted_enemies {
        let dx = enemy.pos.x - player.pos.x;
        let dy = enemy.pos.y - player.pos.y;
        
        let angle_to_enemy = dy.atan2(dx);
        let mut angle_diff = angle_to_enemy - player.a;
        while angle_diff > std::f32::consts::PI { angle_diff -= 2.0 * std::f32::consts::PI; }
        while angle_diff < -std::f32::consts::PI { angle_diff += 2.0 * std::f32::consts::PI; }
        
        if angle_diff.abs() > fov * 0.6 { continue; }
        
        let depth_world = dx * fwd_x + dy * fwd_y;
        if depth_world <= 1e-4 { continue; }
        let depth_cells = depth_world / block_size;
        
        let screen_x = ((angle_diff / (fov * 0.5)) * half_w + half_w) as i32;
        
        // Enemies are full block height
        let sprite_height = (1.0 * proj_plane / depth_cells.max(1e-4)) as i32;
        if sprite_height < 1 { continue; }
        let sprite_width = sprite_height;
        
        let draw_start_y = ((half_h as i32) - sprite_height / 2).max(0);
        let draw_end_y = (draw_start_y + sprite_height).min(screen_h as i32 - 1);
        let draw_start_x = (screen_x - sprite_width / 2).max(0);
        let draw_end_x = (screen_x + sprite_width / 2).min(screen_w as i32 - 1);
        if draw_end_x <= draw_start_x { continue; }
        
        let rs = render_scale.max(1);
        
        // Draw enemy sprite
        for sx in (draw_start_x..=draw_end_x).step_by(rs) {
            let col_index = (sx as usize) / rs;
            
            if col_index < zbuffer.len() && depth_cells > zbuffer[col_index] * 0.98 {
                continue;
            }
            
            let tx = ((((sx - (screen_x - sprite_width / 2)) as f32 / sprite_width as f32)
                * (tex_w as f32 - 1.0)) as usize)
                .min(tex_w.saturating_sub(1));
            
            for sy in (draw_start_y..=draw_end_y).step_by(rs) {
                let ty = ((((sy - draw_start_y) as f32 / sprite_height as f32)
                    * (tex_h as f32 - 1.0)) as usize)
                    .min(tex_h.saturating_sub(1));
                
                let mut color = tex.sample_at(enemy_key, tx, ty);
                
                if color.a < 10 { continue; }
                
                // Muzzle flash effect - brighten enemy when shooting
                if enemy.muzzle_flash > 0 {
                    let flash_intensity = enemy.muzzle_flash as f32 / 5.0;
                    color.r = (color.r as f32 + 100.0 * flash_intensity).min(255.0) as u8;
                    color.g = (color.g as f32 + 80.0 * flash_intensity).min(255.0) as u8;
                    color.b = (color.b as f32 + 20.0 * flash_intensity).min(255.0) as u8;
                }
                
                fog_with_distance(&mut color, depth_cells, 0.08);
                
                if rs > 1 {
                    d.draw_rectangle(sx, sy, rs as i32, rs as i32, color);
                } else {
                    d.draw_pixel(sx, sy, color);
                }
            }
        }
        
        // Draw health bar above enemy
        let health_bar_y = draw_start_y - 15;
        if health_bar_y > 0 {
            let health_bar_width = (sprite_width as f32 * 0.6) as i32;
            let health_bar_height = 4;
            let health_bar_x = screen_x - health_bar_width / 2;
            
            // Background (red)
            d.draw_rectangle(health_bar_x, health_bar_y, health_bar_width, health_bar_height, Color::new(100, 0, 0, 200));
            
            // Current health (green/yellow/red based on percentage)
            let health_percentage = enemy.health as f32 / enemy.max_health as f32;
            let current_health_width = (health_bar_width as f32 * health_percentage) as i32;
            
            let health_color = if health_percentage > 0.6 {
                Color::new(0, 200, 0, 220)
            } else if health_percentage > 0.3 {
                Color::new(255, 165, 0, 220)
            } else {
                Color::new(255, 0, 0, 220)
            };
            
            d.draw_rectangle(health_bar_x, health_bar_y, current_health_width, health_bar_height, health_color);
            
            // Border
            d.draw_rectangle_lines(health_bar_x - 1, health_bar_y - 1, health_bar_width + 2, health_bar_height + 2, Color::BLACK);
        }
    }
}
