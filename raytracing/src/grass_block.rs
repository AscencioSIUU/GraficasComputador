//! Bloque de césped especial con texturas diferentes por cara.

use crate::solid_block::SolidBlock;
use crate::math::Vec3;
use crate::ray::Ray;
use crate::materials::{Intersectable, MaterialParams};

/// Bloque de césped con grass_top arriba y grass_side en los lados
pub struct GrassBlock<'a> {
    pub inner: SolidBlock,
    
    // Textura para la cara superior (grass_top)
    pub top_pixels: &'a [u8],
    pub top_w: u32,
    pub top_h: u32,
    
    // Textura para las caras laterales (grass_side)
    pub side_pixels: &'a [u8],
    pub side_w: u32,
    pub side_h: u32,

    // Parámetros de material
    pub specular_strength: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub ior: f32,
    pub emissive: Vec3,
}

impl<'a> GrassBlock<'a> {
    pub fn new(
        inner: SolidBlock,
        top_pixels: &'a [u8],
        top_w: u32,
        top_h: u32,
        side_pixels: &'a [u8],
        side_w: u32,
        side_h: u32,
        specular_strength: f32,
        shininess: f32,
        reflectivity: f32,
        transparency: f32,
        ior: f32,
        emissive: Vec3,
    ) -> Self {
        Self {
            inner,
            top_pixels,
            top_w,
            top_h,
            side_pixels,
            side_w,
            side_h,
            specular_strength,
            shininess,
            reflectivity,
            transparency,
            ior,
            emissive,
        }
    }

    fn sample_texture(&self, pixels: &[u8], w: u32, h: u32, u: f32, v: f32) -> Vec3 {
        // Sin repetir: un tile por cara
        let uu = u.clamp(0.0, 1.0 - f32::EPSILON);
        let vv = v.clamp(0.0, 1.0 - f32::EPSILON);

        let px = ((uu * w as f32).floor() as u32).min(w - 1);
        let py = (((1.0 - vv) * h as f32).floor() as u32).min(h - 1);

        let idx = ((py * w + px) * 4) as usize;
        if idx + 3 >= pixels.len() {
            return self.inner.albedo_color;
        }
        let r = pixels[idx] as f32 / 255.0;
        let g = pixels[idx + 1] as f32 / 255.0;
        let b = pixels[idx + 2] as f32 / 255.0;
        Vec3::new(r, g, b)
    }


    fn uv_from_point(&self, p: Vec3) -> (f32, f32, bool) {
        let n = self.inner.normal_at(p);
        let min = self.inner.min;
        let max = self.inner.max;
        let dx = max.x - min.x;
        let dy = max.y - min.y;
        let dz = max.z - min.z;

        // Detectar si es cara superior (n.y > 0.5)
        let is_top = n.y > 0.5;

        // Factor de repetición: como la textura es 16x16 y el bloque es ~1.0
        // queremos que NO se estire, sino que se vea como un solo pixel art
        // Así que mantenemos u,v en [0,1] pero la textura 16x16 se verá correctamente
        
        if n.x > 0.5 {
            let u = (p.z - min.z) / dz;
            let v = (p.y - min.y) / dy;
            (u, v, false)
        } else if n.x < -0.5 {
            let u = (max.z - p.z) / dz;
            let v = (p.y - min.y) / dy;
            (u, v, false)
        } else if n.y > 0.5 {
            // CARA SUPERIOR - usar grass_top (16x16)
            let u = (p.x - min.x) / dx;
            let v = (max.z - p.z) / dz;
            (u, v, true)
        } else if n.y < -0.5 {
            // Cara inferior
            let u = (p.x - min.x) / dx;
            let v = (p.z - min.z) / dz;
            (u, v, false)
        } else if n.z > 0.5 {
            let u = (max.x - p.x) / dx;
            let v = (p.y - min.y) / dy;
            (u, v, false)
        } else {
            let u = (p.x - min.x) / dx;
            let v = (p.y - min.y) / dy;
            (u, v, false)
        }
    }
}

impl<'a> Intersectable for GrassBlock<'a> {
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        self.inner.intersect(ray)
    }
    
    fn normal_at(&self, point: Vec3) -> Vec3 {
        self.inner.normal_at(point)
    }
    
    fn albedo(&self) -> Vec3 {
        self.inner.albedo_color
    }
    
    fn albedo_at(&self, point: Vec3) -> Vec3 {
        let (u, v, is_top) = self.uv_from_point(point);
        
        if is_top {
            // Usar grass_top para la cara superior
            self.sample_texture(self.top_pixels, self.top_w, self.top_h, u, v)
        } else {
            // Usar grass_side para las caras laterales
            self.sample_texture(self.side_pixels, self.side_w, self.side_h, u, v)
        }
    }
    
    fn center(&self) -> Vec3 {
        Vec3::new(
            (self.inner.min.x + self.inner.max.x) * 0.5,
            (self.inner.min.y + self.inner.max.y) * 0.5,
            (self.inner.min.z + self.inner.max.z) * 0.5,
        )
    }
    
    fn material_at(&self, p: Vec3) -> MaterialParams {
        MaterialParams {
            albedo: self.albedo_at(p),
            specular_strength: self.specular_strength,
            shininess: self.shininess,
            reflectivity: self.reflectivity,
            transparency: self.transparency,
            ior: self.ior,
            emissive: self.emissive,
            opacity: 1.0,
        }
    }
}
