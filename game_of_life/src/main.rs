mod framebuffer;

use raylib::prelude::*;
use framebuffer::Framebuffer;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
type Grid = [[bool; WIDTH]; HEIGHT];

fn parse_rle(rle: &str, offset_x: usize, offset_y: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let mut x = 0;
    let mut y = 0;
    let mut count = 0;

    for line in rle.lines() {
        // Ignorar encabezado y comentarios
        if line.starts_with('#') || line.starts_with("x") || line.trim().is_empty() {
            continue;
        }

        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch.is_digit(10) {
                // Acumular número (puede tener más de una cifra)
                let mut num_str = ch.to_string();
                while i + 1 < chars.len() && chars[i + 1].is_digit(10) {
                    i += 1;
                    num_str.push(chars[i]);
                }
                count = num_str.parse::<usize>().unwrap();
            } else {
                let run = if count == 0 { 1 } else { count };

                match ch {
                    'b' => x += run,
                    'o' => {
                        for dx in 0..run {
                            result.push((offset_x + x + dx, offset_y + y));
                        }
                        x += run;
                    }
                    '$' => {
                        y += if count == 0 { 1 } else { count };
                        x = 0;
                    }
                    '!' => break,
                    _ => {}
                }

                count = 0;
            }

            i += 1;
        }
    }
    result
}


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

    let rle_data = r#"
        x = 50, y = 50, rule = B3/S23
        18bo$17b3o$12b3o4b2o$11bo2b3o2bob2o$10bo3bobo2bobo$10bo4bobobobob2o$12bo4bobo3b2o$
        4o5bobo4bo3bob3o$o3b2obob3ob2o9b2o$o5b2o5bo$bo2b2obo2bo2bob2o$7bobobobobobo5b4o$
        bo2b2obo2bo2bo2b2obob2o3bo$o5b2o3bobobo3b2o5bo$o3b2obob2o2bo2bo2bob2o2bo$4o5bobobobobobo$
        10b2obo2bo2bob2o2bo$13bo5b2o5bo$b2o9b2ob3obob2o3bo$2b3obo3bo4bobo5b4o$
        2b2o3bobo4bo$2b2obobobobo4bo$5bobo2bobo3bo$4b2obo2b3o2bo$6b2o4b3o$7b3o$8bo!
        "#;

    let pattern = parse_rle(rle_data, 10, 10);

    for (x, y) in pattern {
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
