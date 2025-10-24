use raylib::prelude::*;
use crate::{maze::{Maze, block_size}, player::Player}; // ← agrega block_size

pub fn draw_minimap(d: &mut RaylibDrawHandle, maze: &Maze, player: &Player) {
    let scale = 4;
    // place minimap at the top-left corner
    let origin_x = 0;
    let origin_y = 0;
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            let wall = matches!(c, '#' | 'A' | 'B' | 'C');
            let color = if wall { Color::DARKBLUE } else { Color::DARKGRAY };
            d.draw_rectangle(origin_x + i as i32 * scale,
                             origin_y + j as i32 * scale,
                             scale, scale, color);
        }
    }
    // draw coins on minimap
    for (j, row) in maze.iter().enumerate() {
        for (i, &c) in row.iter().enumerate() {
            if c == 'X' || c == 'x' {
                let cx = origin_x + i as i32 * scale + scale/2;
                let cy = origin_y + j as i32 * scale + scale/2;
                d.draw_circle(cx, cy, (scale/2) as f32, Color::GOLD);
            }
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

/// Draw health bar HUD
pub fn draw_health_hud(d: &mut RaylibDrawHandle, player: &crate::player::Player) {
    let screen_w = d.get_screen_width();
    let screen_h = d.get_screen_height();
    
    // Posición de la barra de vida (esquina inferior izquierda)
    let bar_x = 20;
    let bar_y = screen_h - 60;
    let bar_width = 200;
    let bar_height = 30;
    
    // Borde de la barra
    d.draw_rectangle(bar_x - 2, bar_y - 2, bar_width + 4, bar_height + 4, Color::BLACK);
    
    // Fondo de la barra (rojo oscuro)
    d.draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::new(100, 0, 0, 255));
    
    // Calcular ancho de la barra de vida actual
    let health_percentage = player.health as f32 / player.max_health as f32;
    let current_width = (bar_width as f32 * health_percentage) as i32;
    
    // Color de la barra según el porcentaje de vida
    let health_color = if health_percentage > 0.6 {
        Color::new(0, 200, 0, 255) // Verde
    } else if health_percentage > 0.3 {
        Color::new(255, 165, 0, 255) // Naranja
    } else {
        Color::new(255, 0, 0, 255) // Rojo
    };
    
    // Barra de vida actual
    d.draw_rectangle(bar_x, bar_y, current_width, bar_height, health_color);
    
    // Texto de vida
    let health_text = format!("{} / {}", player.health, player.max_health);
    let text_x = bar_x + bar_width / 2 - 30;
    let text_y = bar_y + 5;
    d.draw_text(&health_text, text_x, text_y, 20, Color::WHITE);
    
    // Etiqueta "HEALTH"
    d.draw_text("HEALTH", bar_x, bar_y - 20, 16, Color::LIGHTGRAY);
}

/// Draw crosshair in the center of the screen
pub fn draw_crosshair(d: &mut RaylibDrawHandle) {
    let screen_w = d.get_screen_width();
    let screen_h = d.get_screen_height();
    let center_x = screen_w / 2;
    let center_y = screen_h / 2;
    
    let crosshair_size = 15;
    let crosshair_thickness = 2;
    let crosshair_gap = 5;
    
    let color = Color::new(0, 255, 0, 200); // Verde semi-transparente
    
    // Línea horizontal izquierda
    d.draw_rectangle(
        center_x - crosshair_size - crosshair_gap,
        center_y - crosshair_thickness / 2,
        crosshair_size,
        crosshair_thickness,
        color
    );
    
    // Línea horizontal derecha
    d.draw_rectangle(
        center_x + crosshair_gap,
        center_y - crosshair_thickness / 2,
        crosshair_size,
        crosshair_thickness,
        color
    );
    
    // Línea vertical superior
    d.draw_rectangle(
        center_x - crosshair_thickness / 2,
        center_y - crosshair_size - crosshair_gap,
        crosshair_thickness,
        crosshair_size,
        color
    );
    
    // Línea vertical inferior
    d.draw_rectangle(
        center_x - crosshair_thickness / 2,
        center_y + crosshair_gap,
        crosshair_thickness,
        crosshair_size,
        color
    );
    
    // Punto central
    d.draw_circle(center_x, center_y, 2.0, color);
}

/// Draw damage overlay (red flash when player takes damage)
pub fn draw_damage_overlay(d: &mut RaylibDrawHandle, player: &crate::player::Player) {
    let health_percentage = player.health as f32 / player.max_health as f32;
    
    // Strong red vignette effect when very low on health
    if health_percentage < 0.3 {
        let alpha = ((1.0 - health_percentage / 0.3) * 120.0) as u8;
        let w = d.get_screen_width();
        let h = d.get_screen_height();
        d.draw_rectangle(0, 0, w, h, Color::new(255, 0, 0, alpha));
    }
    
    // Death screen overlay (fades to black)
    if !player.is_alive() {
        let w = d.get_screen_width();
        let h = d.get_screen_height();
        d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, 200));
        
        // "YOU DIED" message
        let death_text = "YOU DIED";
        let text_size = 60;
        // Use measure_text method instead of unsafe FFI
        let text_width = d.measure_text(death_text, text_size);
        d.draw_text(
            death_text,
            w / 2 - text_width / 2,
            h / 2 - 30,
            text_size,
            Color::RED
        );
    }
}

/// Draw pistol view at bottom center of screen
pub fn draw_pistol_view(d: &mut RaylibDrawHandle, tex: &crate::textures::TextureManager) {
    let screen_w = d.get_screen_width();
    let screen_h = d.get_screen_height();
    
    let pistol_key = "pistol_view";
    let (tex_w, tex_h) = tex.size_of(pistol_key);
    
    // Scale pistol to be about 40% of screen width
    let pistol_display_width = (screen_w as f32 * 0.4) as i32;
    let pistol_display_height = (pistol_display_width as f32 * (tex_h as f32 / tex_w as f32)) as i32;
    
    // Position at bottom center
    let pistol_x = (screen_w - pistol_display_width) / 2;
    let pistol_y = screen_h - pistol_display_height;
    
    // Draw pistol pixel by pixel with transparency
    let scale_x = tex_w as f32 / pistol_display_width as f32;
    let scale_y = tex_h as f32 / pistol_display_height as f32;
    
    for dy in 0..pistol_display_height {
        for dx in 0..pistol_display_width {
            let tx = (dx as f32 * scale_x) as usize;
            let ty = (dy as f32 * scale_y) as usize;
            
            let color = tex.sample_at(pistol_key, tx, ty);
            
            // Skip transparent pixels
            if color.a < 10 {
                continue;
            }
            
            d.draw_pixel(pistol_x + dx, pistol_y + dy, color);
        }
    }
}

/// Draw complete game HUD (health + crosshair + damage overlay + pistol)
pub fn draw_game_hud(d: &mut RaylibDrawHandle, player: &crate::player::Player, tex: &crate::textures::TextureManager) {
    draw_damage_overlay(d, player);
    draw_pistol_view(d, tex);
    draw_health_hud(d, player);
    draw_crosshair(d);
}
