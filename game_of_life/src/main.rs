mod framebuffer;
mod rle;
mod algebra_logic;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use rle::parse_rle;
use algebra_logic::{WIDTH, HEIGHT, Grid, render_grid, step};

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

    let offset_x = (WIDTH - 50) / 2;
    let offset_y = (HEIGHT - 50) / 2;
    let pattern = parse_rle(rle_data, offset_x, offset_y);

    for (x, y) in pattern {
        if x < WIDTH && y < HEIGHT {
            current[y][x] = true;
        }
    }

    while !rl.window_should_close() {
        render_grid(&mut framebuffer, &current);
        framebuffer.swap_buffers(&mut rl, &thread);
        step(&current, &mut next);
        std::mem::swap(&mut current, &mut next);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
