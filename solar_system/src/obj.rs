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
        let mut vertices = Vec::new();

        for i in (0..mesh.positions.len()).step_by(3) {
            vertices.push(Vector3::new(
                mesh.positions[i],
                mesh.positions[i + 1],
                mesh.positions[i + 2],
            ));
        }

        let indices: Vec<usize> = mesh.indices.iter().map(|&i| i as usize).collect();

        Ok(Obj { vertices, indices })
    }

    pub fn get_vertex_array(&self) -> Vec<Vector3> {
        let mut result = Vec::new();
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
