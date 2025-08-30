use std::mem::zeroed;

use crate::{Framebuffer, framebuffer};
use raylib::prelude::*;

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect;
}

pub struct Sphere {
    center: Vector3,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new(center: Vector3, radius: f32, material: Material) -> Sphere {
        Sphere {
            center,
            radius,
            material,
        }
    }

    pub fn center(&self) -> Vector3 {
        self.center
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let oc = *ray_origin - self.center;

        let a = ray_direction.dot(*ray_direction);

        let b = 2.0 * oc.dot(*ray_direction);

        let c = oc.dot(oc) - self.radius * self.radius;

        let discriminant = b * b - 4.0 * a * c;

        let is_intersecting = discriminant > 0.0;

        Intersect::new(self.material, is_intersecting)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse_color: Color,
}

impl Material {
    pub fn new(diffuse_color: Color) -> Self {
        Material { diffuse_color }
    }

    pub fn diffuse_color(&self) -> Color {
        self.diffuse_color
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Intersect {
    pub material: Material,
    pub is_intersecting: bool,
}

impl Intersect {
    pub fn new(material: Material, is_intersecting: bool) -> Self {
        Intersect {
            is_intersecting,
            material,
        }
    }
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere]) {
    let width = framebuffer.width() as f32;
    let height = framebuffer.height() as f32;
    let aspect_ratio = width / height;

    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio;

            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();

            let pixel_color = cast_ray(&Vector3::zero(), &ray_direction, objects);

            framebuffer.set_foreground_color(pixel_color);
            framebuffer.set_pixel(x, y);
        }
    }
}

pub fn cast_ray(ray_origin: &Vector3, ray_direction: &Vector3, objects: &[Sphere]) -> Color {
    for object in objects {
        let intersection = object.ray_intersect(ray_origin, ray_direction);
        if intersection.is_intersecting {
            return intersection.material.diffuse_color;
        }
    }
    Color::new(4, 12, 36, 255)
}
