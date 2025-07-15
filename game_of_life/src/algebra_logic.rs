use crate::framebuffer::Framebuffer;

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 600;
pub type Grid = [[bool; WIDTH]; HEIGHT];

pub fn count_alive_neighbors(grid: &Grid, x: usize, y: usize) -> u8 {
    let mut count = 0;
    for dy in [-1, 0, 1] {
        for dx in [-1, 0, 1] {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = (x as isize + dx).rem_euclid(WIDTH as isize) as usize;
            let ny = (y as isize + dy).rem_euclid(HEIGHT as isize) as usize;
            if grid[ny][nx] {
                count += 1;
            }
        }
    }
    count
}

pub fn step(current: &Grid, next: &mut Grid) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let alive = current[y][x];
            let neighbors = count_alive_neighbors(current, x, y);
            next[y][x] = match (alive, neighbors) {
                (true, 2..=3) => true,
                (false, 3) => true,
                _ => false,
            };
        }
    }
}

pub fn render_grid(framebuffer: &mut Framebuffer, grid: &Grid) {
    framebuffer.clear();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if grid[y][x] {
                framebuffer.set_pixel(x as i32, y as i32);
            }
        }
    }
}
