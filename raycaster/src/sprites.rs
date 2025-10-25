use raylib::prelude::*;
use crate::{player::Player, textures::TextureManager, maze::Maze};
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub pos: Vector2,
    pub health: i32,
    pub max_health: i32,
    pub alive: bool,
    pub shoot_cooldown: i32,
    pub muzzle_flash: i32, // frames remaining for muzzle flash effect
    pub shot_effect_timer: i32, // efecto de disparo visual (como el jugador)
}

impl Enemy {
    pub fn new(pos: Vector2) -> Self {
        Self {
            pos,
            health: 100,
            max_health: 100,
            alive: true,
            shoot_cooldown: 0,
            muzzle_flash: 0,
            shot_effect_timer: 0,
        }
    }
    
    pub fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
        if self.health <= 0 {
            self.alive = false;
        }
    }
    
    /// Move towards player with collision detection
    pub fn move_towards_player(&mut self, player: &Player, maze: &Maze, speed: f32) {
        let dx = player.pos.x - self.pos.x;
        let dy = player.pos.y - self.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance < 1.0 { return; } // Too close, don't move
        
        // Normalize direction
        let dir_x = dx / distance;
        let dir_y = dy / distance;
        
        // Calculate new position
        let new_x = self.pos.x + dir_x * speed;
        let new_y = self.pos.y + dir_y * speed;
        
        // Check collision with walls
        let block_size = crate::maze::block_size() as f32;
        let tile_x = (new_x / block_size) as usize;
        let tile_y = (new_y / block_size) as usize;
        
        let can_move_x = tile_y < maze.len() && tile_x < maze[tile_y].len() 
            && (maze[tile_y][tile_x] == ' ' || maze[tile_y][tile_x] == 'X' || maze[tile_y][tile_x] == 'x' || maze[tile_y][tile_x] == 'P' || maze[tile_y][tile_x] == 'p');
        
        let tile_y2 = (self.pos.y / block_size) as usize;
        let can_move_y = tile_y2 < maze.len() && tile_x < maze[tile_y2].len()
            && (maze[tile_y2][tile_x] == ' ' || maze[tile_y2][tile_x] == 'X' || maze[tile_y2][tile_x] == 'x' || maze[tile_y2][tile_x] == 'P' || maze[tile_y2][tile_x] == 'p');
        
        // Move if no collision
        if can_move_x {
            self.pos.x = new_x;
        }
        
        let tile_x2 = (self.pos.x / block_size) as usize;
        if can_move_y && tile_y < maze.len() && tile_x2 < maze[tile_y].len()
            && (maze[tile_y][tile_x2] == ' ' || maze[tile_y][tile_x2] == 'X' || maze[tile_y][tile_x2] == 'x' || maze[tile_y][tile_x2] == 'P' || maze[tile_y][tile_x2] == 'p') {
            self.pos.y = new_y;
        }
    }
    
    /// Check if enemy can see player (simple line of sight)
    pub fn can_see_player(&self, player: &Player, maze: &Maze) -> bool {
        let dx = player.pos.x - self.pos.x;
        let dy = player.pos.y - self.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance > 500.0 { return false; } // Too far
        
        // Simple raycast to check line of sight
        let steps = (distance / 10.0) as i32;
        let step_x = dx / steps as f32;
        let step_y = dy / steps as f32;
        
        let block_size = crate::maze::block_size() as f32;
        
        for i in 0..steps {
            let check_x = self.pos.x + step_x * i as f32;
            let check_y = self.pos.y + step_y * i as f32;
            
            let tile_x = (check_x / block_size) as usize;
            let tile_y = (check_y / block_size) as usize;
            
            if tile_y >= maze.len() || tile_x >= maze[tile_y].len() {
                return false;
            }
            
            let ch = maze[tile_y][tile_x];
            if ch != ' ' && ch != 'X' && ch != 'x' && ch != 'P' && ch != 'p' {
                return false; // Wall blocking
            }
        }
        
        true
    }
    
    /// Try to shoot at player
    pub fn try_shoot(&mut self, player: &mut Player, maze: &Maze) -> bool {
        if self.shoot_cooldown > 0 {
            return false;
        }
        
        if !self.can_see_player(player, maze) {
            return false;
        }
        
        let dx = player.pos.x - self.pos.x;
        let dy = player.pos.y - self.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Shoot if close enough
        if distance < 400.0 {
            let mut rng = rand::thread_rng();
            let accuracy = 0.7; // 70% accuracy
            
            if rng.gen::<f32>() < accuracy {
                player.take_damage(10); // 10 damage per hit
            }
            
            self.shoot_cooldown = 60; // 1 second cooldown at 60fps
            self.muzzle_flash = 5; // Show flash for 5 frames
            self.shot_effect_timer = 6; // Efecto visual de disparo
            return true;
        }
        
        false
    }
}

pub struct EnemySystem {
    pub list: Vec<Enemy>,
}

impl EnemySystem {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }
    
    /// Spawn enemies from maze (looks for 'E' or random positions)
    pub fn spawn_from_maze(&mut self, maze: &Maze, count: usize) {
        let block_size = crate::maze::block_size() as f32;
        
        // First, collect all valid spawn positions (empty spaces)
        let mut valid_positions = Vec::new();
        for (y, row) in maze.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                // Empty spaces or explicitly marked 'E' for enemy
                if ch == ' ' || ch == 'E' || ch == 'e' {
                    let world_x = (x as f32 + 0.5) * block_size;
                    let world_y = (y as f32 + 0.5) * block_size;
                    valid_positions.push(Vector2::new(world_x, world_y));
                }
            }
        }
        
        // Spawn enemies at random positions
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        valid_positions.shuffle(&mut rng);
        
        for pos in valid_positions.iter().take(count) {
            self.list.push(Enemy::new(*pos));
        }
    }
    
    /// Update enemies (AI, movement, shooting)
    pub fn update(&mut self, player: &mut Player, maze: &Maze) {
        // Remove dead enemies
        self.list.retain(|e| e.alive);
        
        for enemy in &mut self.list {
            // Decrease cooldowns
            if enemy.shoot_cooldown > 0 {
                enemy.shoot_cooldown -= 1;
            }
            if enemy.muzzle_flash > 0 {
                enemy.muzzle_flash -= 1;
            }
            if enemy.shot_effect_timer > 0 {
                enemy.shot_effect_timer -= 1;
            }
            
            let dx = player.pos.x - enemy.pos.x;
            let dy = player.pos.y - enemy.pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            // AI behavior based on distance
            if distance > 150.0 {
                // Move towards player if far away
                enemy.move_towards_player(player, maze, 1.0);
            } else if distance > 100.0 {
                // Stop and shoot if at medium distance
                enemy.try_shoot(player, maze);
            } else {
                // Move slowly and shoot if very close
                enemy.move_towards_player(player, maze, 0.3);
                enemy.try_shoot(player, maze);
            }
        }
    }
    
    /// Get enemy at crosshair (for shooting)
    pub fn get_enemy_at_crosshair(&mut self, player: &Player, max_distance: f32) -> Option<&mut Enemy> {
        let fwd_x = player.a.cos();
        let fwd_y = player.a.sin();
        
        let mut closest_enemy: Option<(usize, f32)> = None;
        
        for (i, enemy) in self.list.iter().enumerate() {
            if !enemy.alive { continue; }
            
            let dx = enemy.pos.x - player.pos.x;
            let dy = enemy.pos.y - player.pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance > max_distance { continue; }
            
            // Check if enemy is in front of player
            let depth = dx * fwd_x + dy * fwd_y;
            if depth <= 0.0 { continue; }
            
            // Check angle (narrow cone for aiming)
            let angle_to_enemy = dy.atan2(dx);
            let mut angle_diff = angle_to_enemy - player.a;
            while angle_diff > std::f32::consts::PI { angle_diff -= 2.0 * std::f32::consts::PI; }
            while angle_diff < -std::f32::consts::PI { angle_diff += 2.0 * std::f32::consts::PI; }
            
            // Aim cone (about 5 degrees)
            if angle_diff.abs() > 0.087 { continue; }
            
            if let Some((_, closest_dist)) = closest_enemy {
                if distance < closest_dist {
                    closest_enemy = Some((i, distance));
                }
            } else {
                closest_enemy = Some((i, distance));
            }
        }
        
        closest_enemy.map(|(i, _)| &mut self.list[i])
    }
}

impl Default for EnemySystem {
    fn default() -> Self {
        Self::new()
    }
}
