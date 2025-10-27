use raylib::prelude::*;
use crate::framebuffer::Framebuffer;
use crate::materials::{MaterialProperties, MaterialType};
use rayon::prelude::*;
use std::path::Path;

// ===================== ImageTexture =====================
#[derive(Clone)]
pub struct ImageTexture {
    pub pixels: Vec<Color>,
    pub width: i32,
    pub height: i32,
}

impl ImageTexture {
    pub fn from_file_or_fallback(path: &str, fallback_color: Color) -> Self {
        if Path::new(path).exists() {
            if let Ok(img) = Image::load_image(path) {
                let width = img.width();
                let height = img.height();
                let pixels = img.get_image_data().to_vec();
                eprintln!("[info] Loaded texture: '{}'", path);
                return Self { pixels, width, height };
            }
        }
        
        // Crear textura procedural simple
        let size = 64;
        let mut pixels = Vec::with_capacity((size * size) as usize);
        for y in 0..size {
            for x in 0..size {
                let checker = ((x / 8) + (y / 8)) % 2;
                let c = if checker == 0 {
                    fallback_color
                } else {
                    Color::new(
                        fallback_color.r.saturating_sub(30),
                        fallback_color.g.saturating_sub(30),
                        fallback_color.b.saturating_sub(30),
                        fallback_color.a,
                    )
                };
                pixels.push(c);
            }
        }
        eprintln!("[info] Created procedural texture for '{}'", path);
        Self { pixels, width: size, height: size }
    }

    pub fn animated_water(time: f32) -> Self {
        let size = 64;
        let mut pixels = Vec::with_capacity((size * size) as usize);
        for y in 0..size {
            for x in 0..size {
                let wave = ((x as f32 * 0.2 + time).sin() + (y as f32 * 0.2 + time * 0.7).cos()) * 0.5;
                let brightness = (128.0 + wave * 50.0) as u8;
                pixels.push(Color::new(
                    brightness / 3,
                    brightness / 2,
                    brightness,
                    200,
                ));
            }
        }
        Self { pixels, width: size, height: size }
    }

    pub fn animated_portal(time: f32) -> Self {
        let size = 64;
        let mut pixels = Vec::with_capacity((size * size) as usize);
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - size as f32 / 2.0;
                let dy = y as f32 - size as f32 / 2.0;
                let dist = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);
                
                let spiral = (dist * 0.3 - angle * 2.0 + time * 3.0).sin();
                let brightness = ((spiral + 1.0) * 127.0) as u8;
                
                pixels.push(Color::new(
                    brightness,
                    brightness / 3,
                    255,
                    255,
                ));
            }
        }
        Self { pixels, width: size, height: size }
    }

    #[inline]
    pub fn sample(&self, u: f32, v: f32) -> Color {
        let uu = u.rem_euclid(1.0);
        let vv = v.rem_euclid(1.0);
        let x = (uu * (self.width - 1) as f32).round().clamp(0.0, (self.width - 1) as f32) as i32;
        let y = ((1.0 - vv) * (self.height - 1) as f32).round().clamp(0.0, (self.height - 1) as f32) as i32;
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        self.pixels[idx]
    }
}

// ===================== Material =====================
#[derive(Clone)]
pub struct Material {
    pub props: MaterialProperties,
    pub texture: Option<ImageTexture>,
    pub material_type: MaterialType,
}

impl Material {
    pub fn new(material_type: MaterialType) -> Self {
        Self {
            props: material_type.properties(),
            texture: None,
            material_type,
        }
    }

    pub fn with_texture(material_type: MaterialType, texture: ImageTexture) -> Self {
        Self {
            props: material_type.properties(),
            texture: Some(texture),
            material_type,
        }
    }

    pub fn get_color(&self, u: f32, v: f32) -> Color {
        if let Some(tex) = &self.texture {
            tex.sample(u, v)
        } else {
            self.props.albedo
        }
    }
}

// ===================== Vector helpers =====================
#[inline] fn v_add(a: Vector3, b: Vector3) -> Vector3 { Vector3::new(a.x+b.x, a.y+b.y, a.z+b.z) }
#[inline] fn v_sub(a: Vector3, b: Vector3) -> Vector3 { Vector3::new(a.x-b.x, a.y-b.y, a.z-b.z) }
#[inline] fn v_mul(a: Vector3, k: f32) -> Vector3 { Vector3::new(a.x*k, a.y*k, a.z*k) }
#[inline] fn v_dot(a: Vector3, b: Vector3) -> f32 { a.x*b.x + a.y*b.y + a.z*b.z }
#[inline] fn v_cross(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.y*b.z - a.z*b.y, a.z*b.x - a.x*b.z, a.x*b.y - a.y*b.x)
}
#[inline] fn v_len(a: Vector3) -> f32 { (a.x*a.x + a.y*a.y + a.z*a.z).sqrt() }
#[inline] fn v_normalize(a: Vector3) -> Vector3 {
    let l = v_len(a);
    if l > 0.0 { Vector3::new(a.x/l, a.y/l, a.z/l) } else { a }
}

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
        if t1 > 0.001 { return Some(t1); }
        let t2 = (-b + sd) / (2.0*a);
        if t2 > 0.001 { return Some(t2); }
        None
    }

    pub fn normal_at(&self, point: Vector3) -> Vector3 {
        v_normalize(v_sub(point, self.center))
    }

    pub fn uv_at(&self, point: Vector3) -> (f32, f32) {
        let p = v_normalize(v_sub(point, self.center));
        let u = 0.5 + p.z.atan2(p.x) / (2.0 * std::f32::consts::PI);
        let v = 0.5 - p.y.asin() / std::f32::consts::PI;
        (u, v)
    }
}

// ===================== Triangle =====================
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
        let p = v_cross(dir, e2);
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
        if t <= 0.001 { return None; }
        let n = v_normalize(v_cross(e1, e2));
        Some((t, n, u, v))
    }

    #[inline]
    pub fn uv_at(&self, u: f32, v: f32) -> (f32, f32) {
        let w = 1.0 - u - v;
        (
            self.uv0.x*w + self.uv1.x*u + self.uv2.x*v,
            self.uv0.y*w + self.uv1.y*u + self.uv2.y*v,
        )
    }
}

// ===================== Cube helpers =====================
#[inline] fn vec2(x:f32, y:f32)->Vector2{ Vector2::new(x,y) }

fn cube_vertices(h: f32) -> [Vector3; 8] {
    [
        Vector3::new(-h, -h, -h), Vector3::new( h, -h, -h),
        Vector3::new( h,  h, -h), Vector3::new(-h,  h, -h),
        Vector3::new(-h, -h,  h), Vector3::new( h, -h,  h),
        Vector3::new( h,  h,  h), Vector3::new(-h,  h,  h),
    ]
}

pub fn cube_triangles(half: f32, mat_top: &Material, mat_side: &Material, mat_bottom: &Material) -> Vec<Triangle> {
    let v = cube_vertices(half);
    let mut tris = Vec::with_capacity(12);

    // front (z=+h)
    tris.push(Triangle::new(v[4], v[5], v[6], vec2(0.0,1.0), vec2(1.0,1.0), vec2(1.0,0.0), mat_side.clone()));
    tris.push(Triangle::new(v[4], v[6], v[7], vec2(0.0,1.0), vec2(1.0,0.0), vec2(0.0,0.0), mat_side.clone()));

    // back (z=-h)
    tris.push(Triangle::new(v[0], v[2], v[1], vec2(0.0,1.0), vec2(1.0,0.0), vec2(1.0,1.0), mat_side.clone()));
    tris.push(Triangle::new(v[0], v[3], v[2], vec2(0.0,1.0), vec2(0.0,0.0), vec2(1.0,0.0), mat_side.clone()));

    // right (x=+h)
    tris.push(Triangle::new(v[1], v[2], v[6], vec2(0.0,1.0), vec2(1.0,1.0), vec2(1.0,0.0), mat_side.clone()));
    tris.push(Triangle::new(v[1], v[6], v[5], vec2(0.0,1.0), vec2(1.0,0.0), vec2(0.0,0.0), mat_side.clone()));

    // left (x=-h)
    tris.push(Triangle::new(v[0], v[7], v[3], vec2(1.0,1.0), vec2(0.0,0.0), vec2(1.0,0.0), mat_side.clone()));
    tris.push(Triangle::new(v[0], v[4], v[7], vec2(1.0,1.0), vec2(0.0,1.0), vec2(0.0,0.0), mat_side.clone()));

    // top (y=+h)
    tris.push(Triangle::new(v[3], v[6], v[2], vec2(0.0,0.0), vec2(1.0,1.0), vec2(1.0,0.0), mat_top.clone()));
    tris.push(Triangle::new(v[3], v[7], v[6], vec2(0.0,0.0), vec2(0.0,1.0), vec2(1.0,1.0), mat_top.clone()));

    // bottom (y=-h)
    tris.push(Triangle::new(v[0], v[1], v[5], vec2(0.0,0.0), vec2(1.0,0.0), vec2(1.0,1.0), mat_bottom.clone()));
    tris.push(Triangle::new(v[0], v[5], v[4], vec2(0.0,0.0), vec2(1.0,1.0), vec2(0.0,1.0), mat_bottom.clone()));

    tris
}

// ===================== Scene =====================
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub triangles: Vec<Triangle>,
    pub sun_dir: Vector3,
    pub sun_color: Color,
    pub ambient: Color,
    pub skybox_top: Color,
    pub skybox_bottom: Color,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            spheres: Vec::new(),
            triangles: Vec::new(),
            sun_dir: v_normalize(Vector3::new(1.0, -1.0, -1.0)),
            sun_color: Color::new(255, 255, 230, 255),
            ambient: Color::new(30, 40, 60, 255),
            skybox_top: Color::new(120, 180, 255, 255),
            skybox_bottom: Color::new(180, 220, 255, 255),
        }
    }

    pub fn add_cube(&mut self, center: Vector3, size: f32, mat_top: Material, mat_side: Material, mat_bottom: Material) {
        let half = size / 2.0;
        let tris = cube_triangles(half, &mat_top, &mat_side, &mat_bottom);
        for mut tri in tris {
            tri.v0 = v_add(tri.v0, center);
            tri.v1 = v_add(tri.v1, center);
            tri.v2 = v_add(tri.v2, center);
            self.triangles.push(tri);
        }
    }

    pub fn update_time(&mut self, time: f32) {
        // Day/night cycle: sol se mueve en círculo
        let angle = time * 0.3;
        let sun_height = angle.sin();
        let sun_forward = angle.cos();
        
        self.sun_dir = v_normalize(Vector3::new(sun_forward, -sun_height, -0.5));
        
        // Color del sol cambia según la hora
        if sun_height > 0.0 {
            // Día
            let brightness = sun_height.powf(0.5);
            self.sun_color = Color::new(
                (255.0 * brightness) as u8,
                (255.0 * brightness) as u8,
                (230.0 * brightness) as u8,
                255,
            );
            self.ambient = Color::new(
                (50.0 + 30.0 * brightness) as u8,
                (60.0 + 40.0 * brightness) as u8,
                (80.0 + 50.0 * brightness) as u8,
                255,
            );
            self.skybox_top = Color::new(
                (50.0 + 70.0 * brightness) as u8,
                (100.0 + 80.0 * brightness) as u8,
                255,
                255,
            );
            self.skybox_bottom = Color::new(
                (150.0 + 30.0 * brightness) as u8,
                (180.0 + 40.0 * brightness) as u8,
                255,
                255,
            );
        } else {
            // Noche
            let darkness = (-sun_height).min(1.0);
            self.sun_color = Color::new(
                (100.0 * (1.0 - darkness)) as u8,
                (100.0 * (1.0 - darkness)) as u8,
                (150.0 * (1.0 - darkness)) as u8,
                255,
            );
            self.ambient = Color::new(
                (10.0 + 20.0 * (1.0 - darkness)) as u8,
                (15.0 + 25.0 * (1.0 - darkness)) as u8,
                (30.0 + 30.0 * (1.0 - darkness)) as u8,
                255,
            );
            self.skybox_top = Color::new(
                (10.0 + 20.0 * (1.0 - darkness)) as u8,
                (15.0 + 30.0 * (1.0 - darkness)) as u8,
                (40.0 + 40.0 * (1.0 - darkness)) as u8,
                255,
            );
            self.skybox_bottom = Color::new(
                (30.0 + 40.0 * (1.0 - darkness)) as u8,
                (35.0 + 50.0 * (1.0 - darkness)) as u8,
                (60.0 + 60.0 * (1.0 - darkness)) as u8,
                255,
            );
        }
    }
}

// ===================== Camera =====================
pub struct Camera {
    pub position: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 5.0, 15.0),
            yaw: 0.0,
            pitch: -0.3,
            distance: 15.0,
        }
    }

    pub fn get_position(&self) -> Vector3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        Vector3::new(x, y + 5.0, z)
    }
}

// ===================== Hit info =====================
struct HitInfo {
    t: f32,
    point: Vector3,
    normal: Vector3,
    material: Material,
    u: f32,
    v: f32,
}

// ===================== Ray tracing =====================
fn cast_ray(scene: &Scene, orig: Vector3, dir: Vector3, depth: u32) -> Color {
    if depth > 3 { return scene.ambient; }

    let mut closest_hit: Option<HitInfo> = None;
    let mut closest_t = f32::INFINITY;

    // Check triangles
    for tri in &scene.triangles {
        if let Some((t, n, bu, bv)) = tri.intersect(orig, dir) {
            if t < closest_t {
                closest_t = t;
                let point = v_add(orig, v_mul(dir, t));
                let (u, v) = tri.uv_at(bu, bv);
                closest_hit = Some(HitInfo {
                    t,
                    point,
                    normal: n,
                    material: tri.mat.clone(),
                    u,
                    v,
                });
            }
        }
    }

    // Check spheres
    for sphere in &scene.spheres {
        if let Some(t) = sphere.intersect(orig, dir) {
            if t < closest_t {
                closest_t = t;
                let point = v_add(orig, v_mul(dir, t));
                let normal = sphere.normal_at(point);
                let (u, v) = sphere.uv_at(point);
                closest_hit = Some(HitInfo {
                    t,
                    point,
                    normal,
                    material: sphere.mat.clone(),
                    u,
                    v,
                });
            }
        }
    }

    if let Some(hit) = closest_hit {
        // Emissive materials
        if hit.material.props.emissive {
            return blend_colors(
                hit.material.get_color(hit.u, hit.v),
                hit.material.props.emission_color,
                hit.material.props.emission_strength,
            );
        }

        let base_color = hit.material.get_color(hit.u, hit.v);
        
        // Lighting
        let to_light = v_mul(scene.sun_dir, -1.0);
        let diffuse = v_dot(hit.normal, to_light).max(0.0);
        
        let mut final_color = blend_colors(base_color, scene.sun_color, diffuse);
        final_color = add_colors(final_color, blend_colors(base_color, scene.ambient, 0.3));

        // Reflection
        if hit.material.props.reflectivity > 0.01 {
            let reflect_dir = reflect(dir, hit.normal);
            let reflect_color = cast_ray(scene, hit.point, reflect_dir, depth + 1);
            final_color = mix_colors(final_color, reflect_color, hit.material.props.reflectivity);
        }

        // Refraction (for transparent materials)
        if hit.material.props.transparency > 0.01 {
            let refract_dir = refract(dir, hit.normal, 1.0, hit.material.props.refractive_index);
            let refract_color = cast_ray(scene, hit.point, refract_dir, depth + 1);
            final_color = mix_colors(final_color, refract_color, hit.material.props.transparency);
        }

        final_color
    } else {
        // Skybox
        let t = (dir.y + 1.0) * 0.5;
        mix_colors(scene.skybox_bottom, scene.skybox_top, t)
    }
}

// ===================== Render =====================
pub fn render_parallel(fb: &mut Framebuffer, scene: &Scene, camera: &Camera) {
    let w = fb.width();
    let h = fb.height();
    let cam_pos = camera.get_position();
    
    let pixels: Vec<(i32, i32, Color)> = (0..h).into_par_iter().flat_map(|y| {
        (0..w).into_par_iter().map(move |x| {
            let ndc_x = 2.0 * (x as f32 + 0.5) / (w as f32) - 1.0;
            let ndc_y = 1.0 - 2.0 * (y as f32 + 0.5) / (h as f32);
            let aspect = w as f32 / h as f32;
            
            let mut dir = Vector3::new(ndc_x * aspect, ndc_y, -1.5);
            dir = rotate_y(dir, camera.yaw);
            dir = rotate_x(dir, camera.pitch);
            dir = v_normalize(dir);
            
            let color = cast_ray(scene, cam_pos, dir, 0);
            (x, y, color)
        })
    }).collect();

    for (x, y, color) in pixels {
        fb.put_pixel(x, y, color);
    }
}

// ===================== Color helpers =====================
fn blend_colors(base: Color, tint: Color, factor: f32) -> Color {
    Color::new(
        ((base.r as f32 * tint.r as f32 / 255.0) * factor).min(255.0) as u8,
        ((base.g as f32 * tint.g as f32 / 255.0) * factor).min(255.0) as u8,
        ((base.b as f32 * tint.b as f32 / 255.0) * factor).min(255.0) as u8,
        base.a,
    )
}

fn add_colors(a: Color, b: Color) -> Color {
    Color::new(
        (a.r as u16 + b.r as u16).min(255) as u8,
        (a.g as u16 + b.g as u16).min(255) as u8,
        (a.b as u16 + b.b as u16).min(255) as u8,
        a.a,
    )
}

fn mix_colors(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        (a.r as f32 * (1.0 - t) + b.r as f32 * t) as u8,
        (a.g as f32 * (1.0 - t) + b.g as f32 * t) as u8,
        (a.b as f32 * (1.0 - t) + b.b as f32 * t) as u8,
        (a.a as f32 * (1.0 - t) + b.a as f32 * t) as u8,
    )
}

fn reflect(incident: Vector3, normal: Vector3) -> Vector3 {
    v_sub(incident, v_mul(normal, 2.0 * v_dot(incident, normal)))
}

fn refract(incident: Vector3, normal: Vector3, n1: f32, n2: f32) -> Vector3 {
    let n = n1 / n2;
    let cos_i = -v_dot(incident, normal);
    let sin_t2 = n * n * (1.0 - cos_i * cos_i);
    
    if sin_t2 > 1.0 {
        // Total internal reflection
        return reflect(incident, normal);
    }
    
    let cos_t = (1.0 - sin_t2).sqrt();
    v_add(v_mul(incident, n), v_mul(normal, n * cos_i - cos_t))
}
