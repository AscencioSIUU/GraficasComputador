//! Primitivas geométricas: triángulos, cubos, esferas.

use crate::math::Vec3;
use crate::ray::Ray;

#[derive(Clone)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub normal: Vec3,
    pub albedo: Vec3,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, albedo: Vec3) -> Self {
        let e1 = v1.sub(v0);
        let e2 = v2.sub(v0);
        let normal = e1.cross(e2).norm();
        Self { v0, v1, v2, normal, albedo }
    }

    /// Intersección Möller–Trumbore.
    pub fn intersect(&self, ray: &Ray) -> Option<f32> {
        const EPSILON: f32 = 1e-6;
        let e1 = self.v1.sub(self.v0);
        let e2 = self.v2.sub(self.v0);
        let h = ray.dir.cross(e2);
        let a = e1.dot(h);
        
        if a.abs() < EPSILON {
            return None;
        }
        
        let f = 1.0 / a;
        let s = ray.orig.sub(self.v0);
        let u = f * s.dot(h);
        
        if u < 0.0 || u > 1.0 {
            return None;
        }
        
        let q = s.cross(e1);
        let v = f * ray.dir.dot(q);
        
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        
        let t = f * e2.dot(q);
        if t > EPSILON {
            Some(t)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub albedo: Vec3,
    pub emissive: Vec3,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, albedo: Vec3, emissive: Vec3) -> Self {
        Self { center, radius, albedo, emissive }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<f32> {
        let oc = ray.orig.sub(self.center);
        let a = ray.dir.dot(ray.dir);
        let b = 2.0 * oc.dot(ray.dir);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant < 0.0 {
            None
        } else {
            let t1 = (-b - discriminant.sqrt()) / (2.0 * a);
            let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
            
            if t1 > 0.001 {
                Some(t1)
            } else if t2 > 0.001 {
                Some(t2)
            } else {
                None
            }
        }
    }

    pub fn normal_at(&self, point: Vec3) -> Vec3 {
        point.sub(self.center).norm()
    }
}

/// Genera 12 triángulos para un cubo centrado con tamaño dado.
pub fn cube_triangles(center: Vec3, size: f32, albedo: Vec3) -> Vec<Triangle> {
    let hs = size * 0.5;
    let c = center;
    
    // 8 vértices
    let v = [
        Vec3::new(c.x - hs, c.y - hs, c.z - hs), // 0: left-bottom-back
        Vec3::new(c.x + hs, c.y - hs, c.z - hs), // 1: right-bottom-back
        Vec3::new(c.x + hs, c.y + hs, c.z - hs), // 2: right-top-back
        Vec3::new(c.x - hs, c.y + hs, c.z - hs), // 3: left-top-back
        Vec3::new(c.x - hs, c.y - hs, c.z + hs), // 4: left-bottom-front
        Vec3::new(c.x + hs, c.y - hs, c.z + hs), // 5: right-bottom-front
        Vec3::new(c.x + hs, c.y + hs, c.z + hs), // 6: right-top-front
        Vec3::new(c.x - hs, c.y + hs, c.z + hs), // 7: left-top-front
    ];
    
    vec![
        // Front (+Z)
        Triangle::new(v[4], v[5], v[6], albedo),
        Triangle::new(v[4], v[6], v[7], albedo),
        // Back (-Z)
        Triangle::new(v[1], v[0], v[3], albedo),
        Triangle::new(v[1], v[3], v[2], albedo),
        // Left (-X)
        Triangle::new(v[0], v[4], v[7], albedo),
        Triangle::new(v[0], v[7], v[3], albedo),
        // Right (+X)
        Triangle::new(v[5], v[1], v[2], albedo),
        Triangle::new(v[5], v[2], v[6], albedo),
        // Top (+Y)
        Triangle::new(v[3], v[7], v[6], albedo),
        Triangle::new(v[3], v[6], v[2], albedo),
        // Bottom (-Y)
        Triangle::new(v[0], v[1], v[5], albedo),
        Triangle::new(v[0], v[5], v[4], albedo),
    ]
}
