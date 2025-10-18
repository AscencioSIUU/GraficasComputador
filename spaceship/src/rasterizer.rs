#![allow(dead_code)]

use raylib::math::{Vector2, Vector3};

pub struct Fragment {
    pub position: Vector2,
    pub color: Vector3,
    pub depth: f32,
}

impl Fragment {
    pub fn new(x: f32, y: f32, color: Vector3, depth: f32) -> Self {
        Fragment {
            position: Vector2::new(x, y),
            color,
            depth,
        }
    }
}

pub struct Vertex2D {
    pub position: Vector2,
    pub depth: f32,
    pub normal: Vector3,
}

impl Vertex2D {
    pub fn new(position: Vector2, depth: f32, normal: Vector3) -> Self {
        Vertex2D {
            position,
            depth,
            normal,
        }
    }
}

fn barycentric_coordinates(p_x: f32, p_y: f32, a: &Vertex2D, b: &Vertex2D, c: &Vertex2D) -> (f32, f32, f32) {
    let a_x = a.position.x;
    let b_x = b.position.x;
    let c_x = c.position.x;
    let a_y = a.position.y;
    let b_y = b.position.y;
    let c_y = c.position.y;

    let area = (b_y - c_y) * (a_x - c_x) + (c_x - b_x) * (a_y - c_y);

    if area.abs() < 1e-10 {
        return (-1.0, -1.0, -1.0);
    }
    
    let w = ((b_y - c_y) * (p_x - c_x) + (c_x - b_x) * (p_y - c_y)) / area;
    let v = ((c_y - a_y) * (p_x - c_x) + (a_x - c_x) * (p_y - c_y)) / area;
    let u = 1.0 - w - v;

    (w, v, u)
}

pub fn triangle(v1: &Vertex2D, v2: &Vertex2D, v3: &Vertex2D) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    let a_x = v1.position.x;
    let b_x = v2.position.x;
    let c_x = v3.position.x;
    let a_y = v1.position.y;
    let b_y = v2.position.y;
    let c_y = v3.position.y;

    let min_x = a_x.min(b_x).min(c_x).floor() as i32;
    let min_y = a_y.min(b_y).min(c_y).floor() as i32;
    let max_x = a_x.max(b_x).max(c_x).ceil() as i32;
    let max_y = a_y.max(b_y).max(c_y).ceil() as i32;

    let normal = v1.normal;
    let normal_len = (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z).sqrt();
    let normalized_normal = if normal_len > 0.0001 {
        Vector3::new(
            normal.x / normal_len,
            normal.y / normal_len,
            normal.z / normal_len,
        )
    } else {
        Vector3::new(0.0, 0.0, -1.0)
    };
    
    let light_intensity = (-normalized_normal.z).max(0.0) * 0.7 + 0.3;
    let color = Vector3::new(light_intensity, light_intensity, light_intensity);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (w, v, u) = barycentric_coordinates(x as f32 + 0.5, y as f32 + 0.5, v1, v2, v3);

            if w >= 0.0 && v >= 0.0 && u >= 0.0 {
                let depth = w * v1.depth + v * v2.depth + u * v3.depth;

                fragments.push(Fragment::new(
                    x as f32,
                    y as f32,
                    color,
                    depth,
                ));
            }
        }
    }

    fragments
}
