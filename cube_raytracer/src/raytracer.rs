use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use std::path::Path;

// ===================== ImageTexture (Vec<Color>) =====================
#[derive(Clone)]
pub struct ImageTexture {
    pub pixels: Vec<Color>,
    pub width: i32,
    pub height: i32,
}

impl ImageTexture {
    pub fn from_file_or_checker(path: &str) -> Self {
        if Path::new(path).exists() {
            if let Ok(img) = Image::load_image(path) {
                let width = img.width();
                let height = img.height();
                let pixels = img.get_image_data().to_vec();
                return Self { pixels, width, height };
            } else {
                eprintln!("[warn] Failed to decode '{}', fallback checker.", path);
            }
        } else {
            eprintln!("[warn] File '{}' not found.", path);
        }

        let alt = "assets/grass_block.png";
        if Path::new(alt).exists() {
            if let Ok(img) = Image::load_image(alt) {
                let width = img.width();
                let height = img.height();
                let pixels = img.get_image_data().to_vec();
                eprintln!("[info] Loaded '{}'.", alt);
                return Self { pixels, width, height };
            } else {
                eprintln!("[warn] Failed to decode '{}', fallback checker.", alt);
            }
        } else {
            eprintln!("[info] '{}' not found. Using checker.", alt);
        }

        // fallback checker 256x256
        let size = 256;
        let tiles = 8;
        let tile = size / tiles;
        let mut pixels = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let cx = x / tile;
                let cy = y / tile;
                let c = if (cx + cy) % 2 == 0 {
                    Color::new(50, 200, 50, 255)
                } else {
                    Color::new(200, 50, 50, 255)
                };
                pixels.push(c);
            }
        }
        eprintln!("[info] Using procedural checker 256x256 as texture.");
        Self { pixels, width: size as i32, height: size as i32 }
    }

    #[inline] fn idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }
    #[inline] pub fn sample(&self, u: f32, v: f32) -> Color {
        let uu = u.rem_euclid(1.0);
        let vv = v.rem_euclid(1.0);
        let x = (uu * (self.width - 1) as f32).round()
            .clamp(0.0, (self.width - 1) as f32) as i32;
        let y = (((1.0 - vv) * (self.height - 1) as f32).round())
            .clamp(0.0, (self.height - 1) as f32) as i32;
        self.pixels[self.idx(x, y)]
    }
}

// ===================== Material =====================
#[derive(Clone)]
pub struct Material {
    pub base_color: Color,
    pub texture: Option<ImageTexture>,
}
impl Material {
    pub fn new(color: Color) -> Self { Self { base_color: color, texture: None } }
    pub fn with_texture(color: Color, texture: ImageTexture) -> Self {
        Self { base_color: color, texture: Some(texture) }
    }
}

// ===================== Vector helpers =====================
#[inline] fn v_add(a: Vector3, b: Vector3) -> Vector3 { Vector3::new(a.x+b.x, a.y+b.y, a.z+b.z) }
#[inline] fn v_sub(a: Vector3, b: Vector3) -> Vector3 { Vector3::new(a.x-b.x, a.y-b.y, a.z-b.z) }
#[inline] fn v_mul(a: Vector3, k: f32) -> Vector3 { Vector3::new(a.x*k, a.y*k, a.z*k) }
#[inline] fn v_dot(a: Vector3, b: Vector3) -> f32 { (&a).dot(b) }
#[inline] fn v_cross(a: Vector3, b: Vector3) -> Vector3 { (&a).cross(b) }
#[inline] fn v_len(a: Vector3) -> f32 { (a.x*a.x + a.y*a.y + a.z*a.z).sqrt() }
#[inline] fn v_normalize(a: Vector3) -> Vector3 { let l = v_len(a); Vector3::new(a.x/l, a.y/l, a.z/l) }

#[inline]
pub fn rotate_y(v: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    Vector3::new(c*v.x + s*v.z, v.y, -s*v.x + c*v.z)
}
#[inline]
pub fn rotate_x(v: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    Vector3::new(v.x, c*v.y - s*v.z, s*v.y + c*v.z)
}

#[inline]
pub fn rotate_z(v: Vector3, angle: f32) -> Vector3 {
    let (s, c) = angle.sin_cos();
    Vector3::new(c*v.x - s*v.y, s*v.x + c*v.y, v.z)
}

/// Yaw (Y), Pitch (X), Roll (Z) applied in order Y -> X -> Z
#[inline]
pub fn rotate_euler(v: Vector3, yaw_y: f32, pitch_x: f32, roll_z: f32) -> Vector3 {
    let r = rotate_y(v, yaw_y);
    let r = rotate_x(r, pitch_x);
    rotate_z(r, roll_z)
}


// ===================== Sphere =====================
#[derive(Clone)]
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
        let oc = v_sub(orig, self.center);
        let a = v_dot(dir, dir);
        let b = 2.0 * v_dot(oc, dir);
        let c = v_dot(oc, oc) - self.radius * self.radius;
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

// ===================== Triangle (UV) =====================
#[derive(Clone)]
pub struct Triangle {
    pub v0: Vector3, pub v1: Vector3, pub v2: Vector3,
    pub uv0: Vector2, pub uv1: Vector2, pub uv2: Vector2,
    pub mat: Material,
}
impl Triangle {
    pub fn new(v0: Vector3, v1: Vector3, v2: Vector3,
               uv0: Vector2, uv1: Vector2, uv2: Vector2,
               mat: Material) -> Self {
        Self { v0, v1, v2, uv0, uv1, uv2, mat }
    }
    pub fn intersect(&self, orig: Vector3, dir: Vector3) -> Option<(f32, Vector3, f32, f32)> {
        let e1 = v_sub(self.v1, self.v0);
        let e2 = v_sub(self.v2, self.v0);
        let p  = v_cross(dir, e2);
        let det = v_dot(e1, p);
        const EPS: f32 = 1e-6;
        if det.abs() < EPS { return None; }
        let inv_det = 1.0 / det;
        let tvec = v_sub(orig, self.v0);
        let u = v_dot(tvec, p) * inv_det;
        if u < 0.0 || u > 1.0 { return None; }
        let q = v_cross(tvec, e1);
        let v = v_dot(dir, q) * inv_det;
        if v < 0.0 || u + v > 1.0 { return None; }
        let t = v_dot(e2, q) * inv_det;
        if t <= 0.0 { return None; }
        let n = v_normalize(v_cross(e1, e2));
        Some((t, n, u, v))
    }
    #[inline]
    pub fn uv_at(&self, u: f32, v: f32) -> Vector2 {
        let w = 1.0 - u - v;
        Vector2::new(
            self.uv0.x*w + self.uv1.x*u + self.uv2.x*v,
            self.uv0.y*w + self.uv1.y*u + self.uv2.y*v,
        )
    }
}

#[inline] fn vec2(x:f32, y:f32)->Vector2{ Vector2::new(x,y) }

// ===================== Cube (caras estilo Minecraft) =====================
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

/// Usa texturas distintas: top / side / bottom
pub fn cube_local_triangles_minecraft(
    half: f32,
    mat_top: &Material,
    mat_side: &Material,
    mat_bottom: &Material,
) -> Vec<Triangle> {
    let v = cube_vertices(half);
    let mut tris = Vec::with_capacity(12);

    // front (z=+h) -> side
    tris.push(Triangle::new(v[4], v[5], v[6], vec2(0.0,1.0), vec2(1.0,1.0), vec2(1.0,0.0), mat_side.clone()));
    tris.push(Triangle::new(v[4], v[6], v[7], vec2(0.0,1.0), vec2(1.0,0.0), vec2(0.0,0.0), mat_side.clone()));

    // back (z=-h) -> side
    tris.push(Triangle::new(v[0], v[2], v[1], vec2(0.0,1.0), vec2(1.0,0.0), vec2(1.0,1.0), mat_side.clone()));
    tris.push(Triangle::new(v[0], v[3], v[2], vec2(0.0,1.0), vec2(0.0,0.0), vec2(1.0,0.0), mat_side.clone()));

    // right (x=+h) -> side
    tris.push(Triangle::new(v[1], v[2], v[6], vec2(0.0,1.0), vec2(1.0,1.0), vec2(1.0,0.0), mat_side.clone()));
    tris.push(Triangle::new(v[1], v[6], v[5], vec2(0.0,1.0), vec2(1.0,0.0), vec2(0.0,0.0), mat_side.clone()));

    // left (x=-h) -> side
    tris.push(Triangle::new(v[0], v[7], v[3], vec2(1.0,1.0), vec2(0.0,0.0), vec2(1.0,0.0), mat_side.clone()));
    tris.push(Triangle::new(v[0], v[4], v[7], vec2(1.0,1.0), vec2(0.0,1.0), vec2(0.0,0.0), mat_side.clone()));

    // top (y=+h) -> top
    tris.push(Triangle::new(v[3], v[6], v[2], vec2(0.0,0.0), vec2(1.0,1.0), vec2(1.0,0.0), mat_top.clone()));
    tris.push(Triangle::new(v[3], v[7], v[6], vec2(0.0,0.0), vec2(0.0,1.0), vec2(1.0,1.0), mat_top.clone()));

    // bottom (y=-h) -> bottom
    tris.push(Triangle::new(v[0], v[1], v[5], vec2(0.0,0.0), vec2(1.0,0.0), vec2(1.0,1.0), mat_bottom.clone()));
    tris.push(Triangle::new(v[0], v[5], v[4], vec2(0.0,0.0), vec2(1.0,1.0), vec2(0.0,1.0), mat_bottom.clone()));

    tris
}

// ===================== render =====================
pub fn render(fb: &mut Framebuffer, spheres: &[Sphere], tris: &[Triangle]) {
    let w = fb.width();
    let h = fb.height();
    let cam = Vector3::new(0.0, 0.0, 0.0);
    let light_dir = v_normalize(Vector3::new(1.0, -1.0, -1.0));

    for y in 0..h {
        for x in 0..w {
            let ndc_x = 2.0 * (x as f32 + 0.5) / (w as f32) - 1.0;
            let ndc_y = 1.0 - 2.0 * (y as f32 + 0.5) / (h as f32);
            let dir = v_normalize(Vector3::new(ndc_x, ndc_y, -1.5));

            let mut closest_t = f32::INFINITY;
            let mut pixel: Option<Color> = None;

            for t in tris {
                if let Some((tt, n, bu, bv)) = t.intersect(cam, dir) {
                    if tt < closest_t {
                        closest_t = tt;
                        let diff = v_dot(n, v_mul(light_dir, -1.0)).max(0.0);
                        let base = if let Some(tex) = &t.mat.texture {
                            let uv = t.uv_at(bu, bv);
                            tex.sample(uv.x, uv.y)
                        } else { t.mat.base_color };
                        pixel = Some(Color::new(
                            (base.r as f32 * diff) as u8,
                            (base.g as f32 * diff) as u8,
                            (base.b as f32 * diff) as u8,
                            255,
                        ));
                    }
                }
            }

            for s in spheres {
                if let Some(tt) = s.intersect(cam, dir) {
                    if tt < closest_t {
                        closest_t = tt;
                        let hit = v_add(cam, v_mul(dir, tt));
                        let n = v_normalize(v_sub(hit, s.center));
                        let diff = v_dot(n, v_mul(light_dir, -1.0)).max(0.0);
                        let c = s.mat.base_color;
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
