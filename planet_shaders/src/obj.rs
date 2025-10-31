use raylib::math::Vector3;
use std::path::Path;

pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<usize>,
}

impl Obj {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let (models, _materials) = tobj::load_obj(
            path.as_ref(),
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )?;

        if models.is_empty() {
            return Err("No models found in OBJ file".into());
        }

        let mesh = &models[0].mesh;

        if mesh.positions.len() % 3 != 0 {
            return Err("Positions array length is not a multiple of 3".into());
        }
        if mesh.indices.len() % 3 != 0 {
            return Err("Indices array length is not a multiple of 3 (after triangulation)".into());
        }

        // Build raw vertices (as loaded)
        let mut vertices: Vec<Vector3> = Vec::with_capacity(mesh.positions.len() / 3);
        for i in (0..mesh.positions.len()).step_by(3) {
            vertices.push(Vector3::new(
                mesh.positions[i],
                mesh.positions[i + 1],
                mesh.positions[i + 2],
            ));
        }

        // --- Normalize: recenter to bbox center and scale to unit radius ---
        let (min_v, max_v) = bbox(&vertices);
        let center = Vector3::new(
            0.5 * (min_v.x + max_v.x),
            0.5 * (min_v.y + max_v.y),
            0.5 * (min_v.z + max_v.z),
        );
        let mut max_r = 0.0_f32;
        for v in &vertices {
            let dx = v.x - center.x;
            let dy = v.y - center.y;
            let dz = v.z - center.z;
            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            if r > max_r {
                max_r = r;
            }
        }
        let scale = if max_r > 1e-6 { 1.0 / max_r } else { 1.0 };

        for v in &mut vertices {
            v.x = (v.x - center.x) * scale;
            v.y = (v.y - center.y) * scale;
            v.z = (v.z - center.z) * scale;
        }

        // Copy indices as usize
        let mut indices: Vec<usize> = mesh.indices.iter().map(|&i| i as usize).collect();

        // Optional: flip triangle winding if backface culling removes everything.
        // Set to `true` if you still see nothing with culling enabled.
        const REVERSE_WINDING: bool = false;
        if REVERSE_WINDING {
            for i in (0..indices.len()).step_by(3) {
                indices.swap(i + 1, i + 2);
            }
        }

        Ok(Obj { vertices, indices })
    }

    pub fn get_vertex_array(&self) -> Vec<Vector3> {
        // Expand indexed triangles into a flat vertex list (v0,v1,v2, v0,v1,v2, ...)
        let mut result = Vec::with_capacity(self.indices.len());
        for i in (0..self.indices.len()).step_by(3) {
            if i + 2 < self.indices.len() {
                result.push(self.vertices[self.indices[i]]);
                result.push(self.vertices[self.indices[i + 1]]);
                result.push(self.vertices[self.indices[i + 2]]);
            }
        }
        result
    }
}

// Helpers
fn bbox(verts: &[Vector3]) -> (Vector3, Vector3) {
    let mut min_v = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max_v = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for v in verts {
        if v.x < min_v.x { min_v.x = v.x; }
        if v.y < min_v.y { min_v.y = v.y; }
        if v.z < min_v.z { min_v.z = v.z; }
        if v.x > max_v.x { max_v.x = v.x; }
        if v.y > max_v.y { max_v.y = v.y; }
        if v.z > max_v.z { max_v.z = v.z; }
    }
    (min_v, max_v)
}
