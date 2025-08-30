use raylib::prelude::*;
use crate::framebuffer::Framebuffer;

// -------- simple material (diffuse only) --------
#[derive(Clone, Copy)]
pub struct Material {
    pub color: Color,
}
impl Material {
    pub fn new(color: Color) -> Self { Self { color } }
}

// -------- spheres (keep for ground, etc.) --------
#[derive(Clone, Copy)]
pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
    pub mat: Material,
}
impl Sphere {
    pub fn new(center: Vector3, radius: f32, mat: Material) -> Self {
        Self { center, radius, mat }
    }

    pub fn intersect(&self, orig: Vector3, dir: Vector3) -> Option<f32> {
        let oc = sub(orig, self.center);
        let a = dot(dir, dir);
        let b = 2.0 * dot(oc, dir);
        let c = dot(oc, oc) - self.radius * self.radius;
        let disc = b*b - 4.0*a*c;
        if disc < 0.0 { return None; }
        let sd = disc.sqrt();
        let t1 = (-b - sd) / (2.0*a);
        if t1 > 0.0 { return Some(t1); }
        let t2 = (-b + sd) / (2.0*a);
        if t2 > 0.0 { return Some(t2); }
        None
    }
}

// -------- triangles for smooth cube faces --------
#[derive(Clone, Copy)]
pub struct Triangle {
    pub v0: Vector3,
    pub v1: Vector3,
    pub v2: Vector3,
    pub mat: Material,
}
impl Triangle {
    pub fn new(v0: Vector3, v1: Vector3, v2: Vector3, mat: Material) -> Self {
        Self { v0, v1, v2, mat }
    }

    // Möller–Trumbore. Returns (t, geometric normal)
    pub fn intersect(&self, orig: Vector3, dir: Vector3) -> Option<(f32, Vector3)> {
        let e1 = sub(self.v1, self.v0);
        let e2 = sub(self.v2, self.v0);
        let p  = cross(dir, e2);
        let det = dot(e1, p);
        const EPS: f32 = 1e-6;
        if det.abs() < EPS { return None; }
        let inv_det = 1.0 / det;
        let tvec = sub(orig, self.v0);
        let u = dot(tvec, p) * inv_det;
        if u < 0.0 || u > 1.0 { return None; }
        let q = cross(tvec, e1);
        let v = dot(dir, q) * inv_det;
        if v < 0.0 || u + v > 1.0 { return None; }
        let t = dot(e2, q) * inv_det;
        if t <= 0.0 { return None; }
        let n = normalize(cross(e1, e2)); // flat normal
        Some((t, n))
    }
}

// -------- vector helpers (no method calls) --------
#[inline] fn dot(a: Vector3, b: Vector3) -> f32 { a.x*b.x + a.y*b.y + a.z*b.z }
#[inline] fn cross(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.y*b.z - a.z*b.y, a.z*b.x - a.x*b.z, a.x*b.y - a.y*b.x)
}
#[inline] fn len(v: Vector3) -> f32 { (v.x*v.x + v.y*v.y + v.z*v.z).sqrt() }
#[inline] fn normalize(v: Vector3) -> Vector3 { let l = len(v); Vector3::new(v.x/l, v.y/l, v.z/l) }
#[inline] fn add(a: Vector3, b: Vector3) -> Vector3 { Vector3::new(a.x+b.x, a.y+b.y, a.z+b.z) }
#[inline] fn sub(a: Vector3, b: Vector3) -> Vector3 { Vector3::new(a.x-b.x, a.y-b.y, a.z-b.z) }
#[inline] fn mul(v: Vector3, k: f32) -> Vector3 { Vector3::new(v.x*k, v.y*k, v.z*k) }
#[inline] pub fn rotate_y(v: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    Vector3::new(c*v.x + s*v.z, v.y, -s*v.x + c*v.z)
}

// -------- cube helpers --------
// Local (centered at origin) cube vertices with half-extent `h`
fn cube_vertices(h: f32) -> [Vector3; 8] {
    [
        Vector3::new(-h, -h, -h), // 0
        Vector3::new( h, -h, -h), // 1
        Vector3::new( h,  h, -h), // 2
        Vector3::new(-h,  h, -h), // 3
        Vector3::new(-h, -h,  h), // 4
        Vector3::new( h, -h,  h), // 5
        Vector3::new( h,  h,  h), // 6
        Vector3::new(-h,  h,  h), // 7
    ]
}

// Return 12 triangles (two per face) using the vertex indices above
pub fn cube_local_triangles(half: f32) -> Vec<[Vector3; 3]> {
    let v = cube_vertices(half);
    let idx: [[usize; 3]; 12] = [
        // front  (z = +h)
        [4, 5, 6], [4, 6, 7],
        // back   (z = -h)
        [0, 2, 1], [0, 3, 2],
        // right  (x = +h)
        [1, 2, 6], [1, 6, 5],
        // left   (x = -h)
        [0, 7, 3], [0, 4, 7],
        // top    (y = +h)
        [3, 6, 2], [3, 7, 6],
        // bottom (y = -h)
        [0, 1, 5], [0, 5, 4],
    ];
    idx.iter().map(|[a,b,c]| [v[*a], v[*b], v[*c]]).collect()
}

// -------- renderer (spheres + triangles) --------
pub fn render(fb: &mut Framebuffer, spheres: &[Sphere], tris: &[Triangle]) {
    let w = fb.width();
    let h = fb.height();
    let cam = Vector3::new(0.0, 0.0, 0.0);
    let light_dir = normalize(Vector3::new(1.0, -1.0, -1.0));

    for y in 0..h {
        for x in 0..w {
            let ndc_x = 2.0 * (x as f32 + 0.5) / (w as f32) - 1.0;
            let ndc_y = 1.0 - 2.0 * (y as f32 + 0.5) / (h as f32);
            let dir = normalize(Vector3::new(ndc_x, ndc_y, -1.5));

            let mut closest_t = f32::INFINITY;
            let mut pixel: Option<Color> = None;

            // triangles first (the cube)
            for t in tris {
                if let Some((tt, n)) = t.intersect(cam, dir) {
                    if tt < closest_t {
                        closest_t = tt;
                        let diff = dot(n, -light_dir).max(0.0);
                        let c = t.mat.color;
                        pixel = Some(Color::new(
                            (c.r as f32 * diff) as u8,
                            (c.g as f32 * diff) as u8,
                            (c.b as f32 * diff) as u8,
                            255,
                        ));
                    }
                }
            }

            // spheres (e.g., ground)
            for s in spheres {
                if let Some(tt) = s.intersect(cam, dir) {
                    if tt < closest_t {
                        closest_t = tt;
                        let hit = add(cam, mul(dir, tt));
                        let n = normalize(sub(hit, s.center));
                        let diff = dot(n, -light_dir).max(0.0);
                        let c = s.mat.color;
                        pixel = Some(Color::new(
                            (c.r as f32 * diff) as u8,
                            (c.g as f32 * diff) as u8,
                            (c.b as f32 * diff) as u8,
                            255,
                        ));
                    }
                }
            }

            if let Some(col) = pixel {
                fb.set_foreground_color(col);
                fb.set_pixel(x, y);
            }
        }
    }
}
