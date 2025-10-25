use std::{fs, path::Path};

pub type Maze = Vec<Vec<char>>;

pub fn load_maze<P: AsRef<Path>>(path: P) -> Maze {
    let s = fs::read_to_string(path).expect("map file");
    s.lines().map(|l| l.chars().collect()).collect()
}

pub fn block_size() -> usize { 64 } // tamaño del bloque en pixels

pub fn is_wall(maze: &Maze, i: isize, j: isize) -> bool {
    if i < 0 || j < 0 { return true; }
    let (i, j) = (i as usize, j as usize);
    if j >= maze.len() || i >= maze[j].len() { return true; }
    let ch = maze[j][i];
    // Espacios, jugador (P/p) y monedas (X/x) no son paredes
    ch != ' ' && ch != 'P' && ch != 'p' && ch != 'X' && ch != 'x'
}

/// Sistema de fog of war para el mapa
pub struct FogOfWar {
    explored: Vec<Vec<f32>>, // 0.0 = no explorado, 1.0 = completamente explorado
    width: usize,
    height: usize,
}

impl FogOfWar {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            explored: vec![vec![0.0; width]; height],
            width,
            height,
        }
    }

    /// Actualizar fog of war basado en la posición del jugador
    pub fn update(&mut self, player_pos: raylib::prelude::Vector2, maze: &Maze, visibility_radius: f32) {
        let block = block_size() as f32;
        let player_tile_x = (player_pos.x / block) as i32;
        let player_tile_y = (player_pos.y / block) as i32;
        
        let radius_tiles = (visibility_radius / block) as i32;
        
        for dy in -radius_tiles..=radius_tiles {
            for dx in -radius_tiles..=radius_tiles {
                let tx = player_tile_x + dx;
                let ty = player_tile_y + dy;
                
                if tx < 0 || ty < 0 || ty >= self.height as i32 || tx >= self.width as i32 {
                    continue;
                }
                
                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                if distance > radius_tiles as f32 {
                    continue;
                }
                
                // Verificar line of sight simple
                if self.has_line_of_sight(player_tile_x, player_tile_y, tx, ty, maze) {
                    let exploration = 1.0 - (distance / radius_tiles as f32).min(1.0);
                    let current = self.explored[ty as usize][tx as usize];
                    self.explored[ty as usize][tx as usize] = current.max(exploration * 0.3 + 0.7);
                }
            }
        }
    }
    
    /// Verificar si hay línea de visión entre dos puntos
    fn has_line_of_sight(&self, x0: i32, y0: i32, x1: i32, y1: i32, maze: &Maze) -> bool {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        
        let mut x = x0;
        let mut y = y0;
        
        for _ in 0..100 {
            if x == x1 && y == y1 {
                return true;
            }
            
            if x < 0 || y < 0 || y >= maze.len() as i32 || x >= maze[0].len() as i32 {
                return false;
            }
            
            let ch = maze[y as usize][x as usize];
            if ch != ' ' && ch != 'X' && ch != 'x' && ch != 'P' && ch != 'p' && !(x == x0 && y == y0) {
                return false;
            }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
        
        false
    }
    
    /// Obtener nivel de exploración de una celda (0.0 a 1.0)
    pub fn get_exploration(&self, x: usize, y: usize) -> f32 {
        if y < self.height && x < self.width {
            self.explored[y][x]
        } else {
            0.0
        }
    }
}
