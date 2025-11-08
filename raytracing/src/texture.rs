//! Sistema de texturas RGBA8 cargadas desde archivos.

use crate::math::Vec3;

pub struct Tex<'a> {
    pub pix: &'a [u8],
    pub w: u32,
    pub h: u32,
}

impl<'a> Tex<'a> {
    /// Muestrea la textura en coordenadas UV [0,1] con wrapping.
    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let uu = u.fract();
        let vv = v.fract();
        let uu = if uu < 0.0 { uu + 1.0 } else { uu };
        let vv = if vv < 0.0 { vv + 1.0 } else { vv };
        
        let px = (uu * (self.w as f32 - 1.0)).round() as u32;
        let py = ((1.0 - vv) * (self.h as f32 - 1.0)).round() as u32;
        let idx = ((py * self.w + px) * 4) as usize;
        
        if idx + 3 >= self.pix.len() {
            return Vec3::new(1.0, 0.0, 1.0); // Magenta = error
        }
        
        Vec3::new(
            self.pix[idx] as f32 / 255.0,
            self.pix[idx + 1] as f32 / 255.0,
            self.pix[idx + 2] as f32 / 255.0,
        )
    }
}

/// Carga una imagen como bytes RGBA8 junto con sus dimensiones.
pub fn load_texture_rgba(path: &str) -> Option<(Vec<u8>, u32, u32)> {
    if let Ok(img) = image::open(path) {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        Some((rgba.into_raw(), w, h))
    } else {
        None
    }
}
