//! Cámara orbital que genera rayos primarios para el raytracer.

use crate::math::Vec3;
use crate::ray::Ray;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y: f32,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3, up: Vec3, fov_y: f32) -> Self {
        Self { eye, target, up, fov_y }
    }

    /// Genera un rayo que atraviesa el píxel definido por `(u, v)` en NDC [0,1].
    pub fn make_ray(&self, u: f32, v: f32, aspect: f32) -> Ray {
        let fov = self.fov_y.to_radians();
        let scale = (fov * 0.5).tan();
        let forward = self.target.sub(self.eye).norm();
        let right = forward.cross(self.up).norm();
        let up = right.cross(forward).norm();
        let x = (2.0 * u - 1.0) * aspect * scale;
        let y = (1.0 - 2.0 * v) * scale;
        let dir = right.mul(x).add(up.mul(y)).add(forward).norm();
        Ray {
            orig: self.eye,
            dir,
        }
    }
}

/// Controlador de cámara orbital simple.
pub struct OrbitCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
    pub target: Vec3,
}

impl OrbitCamera {
    pub fn new(yaw: f32, pitch: f32, radius: f32, target: Vec3) -> Self {
        Self { yaw, pitch, radius, target }
    }

    pub fn get_position(&self) -> Vec3 {
        let x = self.radius * self.yaw.sin() * self.pitch.cos();
        let y = self.radius * self.pitch.sin();
        let z = self.radius * self.yaw.cos() * self.pitch.cos();
        Vec3::new(x, y, z).add(self.target)
    }

    pub fn to_camera(&self, fov_y: f32) -> Camera {
        Camera::new(
            self.get_position(),
            self.target,
            Vec3::new(0.0, 1.0, 0.0),
            fov_y,
        )
    }
}
