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
