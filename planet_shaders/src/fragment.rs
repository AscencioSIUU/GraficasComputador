use raylib::math::{Vector2, Vector3};
use crate::shader::Uniforms;

pub struct Fragment {
    pub position: Vector2,
    pub color: Vector3,
    pub depth: f32,
}

pub struct Vertex2D {
    pub position: Vector2,
    pub depth: f32,
    pub normal: Vector3,
    pub world_pos: Vector3,  // Nueva: posición en espacio mundo para UV
}

impl Vertex2D {
    pub fn new(position: Vector2, depth: f32, normal: Vector3, world_pos: Vector3) -> Self {
        Vertex2D { position, depth, normal, world_pos }
    }
}

fn barycentric_coordinates(p_x: f32, p_y: f32, a: &Vertex2D, b: &Vertex2D, c: &Vertex2D) -> (f32, f32, f32) {
    let area = (b.position.y - c.position.y) * (a.position.x - c.position.x) 
             + (c.position.x - b.position.x) * (a.position.y - c.position.y);

    if area.abs() < 1e-10 {
        return (-1.0, -1.0, -1.0);
    }
    
    let w = ((b.position.y - c.position.y) * (p_x - c.position.x) 
           + (c.position.x - b.position.x) * (p_y - c.position.y)) / area;
    let v = ((c.position.y - a.position.y) * (p_x - c.position.x) 
           + (a.position.x - c.position.x) * (p_y - c.position.y)) / area;
    let u = 1.0 - w - v;

    (w, v, u)
}

pub fn triangle(v1: &Vertex2D, v2: &Vertex2D, v3: &Vertex2D, uniforms: &Uniforms) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    let min_x = v1.position.x.min(v2.position.x).min(v3.position.x).floor() as i32;
    let min_y = v1.position.y.min(v2.position.y).min(v3.position.y).floor() as i32;
    let max_x = v1.position.x.max(v2.position.x).max(v3.position.x).ceil() as i32;
    let max_y = v1.position.y.max(v2.position.y).max(v3.position.y).ceil() as i32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (w, v, u) = barycentric_coordinates(x as f32 + 0.5, y as f32 + 0.5, v1, v2, v3);

            if w >= 0.0 && v >= 0.0 && u >= 0.0 {
                let depth = w * v1.depth + v * v2.depth + u * v3.depth;
                
                // Interpolar posición en espacio mundo y normal
                let world_pos = Vector3::new(
                    w * v1.world_pos.x + v * v2.world_pos.x + u * v3.world_pos.x,
                    w * v1.world_pos.y + v * v2.world_pos.y + u * v3.world_pos.y,
                    w * v1.world_pos.z + v * v2.world_pos.z + u * v3.world_pos.z,
                );
                
                let normal = Vector3::new(
                    w * v1.normal.x + v * v2.normal.x + u * v3.normal.x,
                    w * v1.normal.y + v * v2.normal.y + u * v3.normal.y,
                    w * v1.normal.z + v * v2.normal.z + u * v3.normal.z,
                );
                
                // Aplicar shader
                let color = uniforms.fragment_shader(world_pos, normal);
                
                fragments.push(Fragment { 
                    position: Vector2::new(x as f32, y as f32), 
                    color, 
                    depth 
                });
            }
        }
    }

    fragments
}
