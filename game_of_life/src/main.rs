mod framebuffer;

use raylib::prelude::*;
use framebuffer::Framebuffer;

const WIDTH: usize = 100;
const HEIGHT: usize = 100;
type Grid = [[bool; WIDTH]; HEIGHT];

fn count_alive_neighbors(grid: &Grid, x: usize, y: usize) -> u8 {
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

fn step(current: &Grid, next: &mut Grid) {
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

fn render_grid(framebuffer: &mut Framebuffer, grid: &Grid) {
    framebuffer.clear();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if grid[y][x] {
                framebuffer.set_pixel(x as i32, y as i32);
            }
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;

    let (mut rl, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Game of Life")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(
        WIDTH as i32,
        HEIGHT as i32,
        Color::new(50, 50, 100, 255),
    );

    let mut current: Grid = [[false; WIDTH]; HEIGHT];
    let mut next: Grid = [[false; WIDTH]; HEIGHT];

    let cruz = vec![
        (50, 48),
        (50, 49),
        (50, 50),
        (50, 51),
        (50, 52),
        (48, 50),
        (49, 50),
        (51, 50),
        (52, 50),
    ];
    for (x, y) in cruz {
        current[y][x] = true;
    }

    while !rl.window_should_close() {
        render_grid(&mut framebuffer, &current);
        framebuffer.swap_buffers(&mut rl, &thread);
        step(&current, &mut next);
        std::mem::swap(&mut current, &mut next);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
