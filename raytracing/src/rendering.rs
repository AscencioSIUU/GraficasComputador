//! Motor de renderizado: raytracing + rasterización híbrida.

use crate::camera::Camera;
use crate::math::Vec3;
use crate::ray::Ray;
use crate::scene::Scene;
use rayon::prelude::*;

const MAX_T: f32 = 1000.0;
const MIN_T: f32 = 0.001;

/// Traza un rayo y devuelve el color final.
pub fn trace_ray(ray: &Ray, scene: &Scene, depth: u32, sun_brightness: f32) -> Vec3 {
    if depth > 2 {
        return scene.ambient;
    }

    let mut closest_t = MAX_T;
    let mut hit_color = None;

    // Intersección con triángulos
    for tri in &scene.triangles {
        if let Some(t) = tri.intersect(ray) {
            if t > MIN_T && t < closest_t {
                closest_t = t;
                hit_color = Some(tri.albedo);
            }
        }
    }

    // Intersección con esferas (con emisión)
    for sphere in &scene.spheres {
        if let Some(t) = sphere.intersect(ray) {
            if t > MIN_T && t < closest_t {
                closest_t = t;
                // Si es emisiva (como el sol o antorcha), mezclar albedo + emisión
                let color = sphere.albedo.add(sphere.emissive);
                hit_color = Some(color);
            }
        }
    }

    if let Some(color) = hit_color {
        // Iluminación modulada por brillo del sol
        let light = scene.ambient.add(Vec3::new(0.6, 0.6, 0.6).mul(sun_brightness));
        color.hadamard(light).clamp(0.0, 1.0)
    } else {
        // Color de cielo modulado por hora del día
        scene.sky_color.mul(sun_brightness * 0.5 + 0.5)
    }
}

/// Renderiza la escena con raytracing paralelo.
pub fn render_raytracing(
    frame: &mut [u8],
    width: u32,
    height: u32,
    camera: &Camera,
    scene: &Scene,
    sun_brightness: f32,
) {
    let aspect = width as f32 / height as f32;
    
    frame.par_chunks_mut(4).enumerate().for_each(|(i, pixel)| {
        let x = (i as u32) % width;
        let y = (i as u32) / width;
        
        let u = (x as f32 + 0.5) / width as f32;
        let v = (y as f32 + 0.5) / height as f32;
        
        let ray = camera.make_ray(u, v, aspect);
        let color = trace_ray(&ray, scene, 0, sun_brightness);
        
        pixel[0] = (color.x * 255.0) as u8;
        pixel[1] = (color.y * 255.0) as u8;
        pixel[2] = (color.z * 255.0) as u8;
        pixel[3] = 255;
    });
}
