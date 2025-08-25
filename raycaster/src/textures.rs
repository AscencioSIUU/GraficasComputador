use std::collections::HashMap;
use image::{GenericImageView, RgbaImage};
use raylib::prelude::Color;

/// Simple texture manager: keeps RGBA images in memory and lets you sample pixels.
pub struct TextureManager {
    atlas: HashMap<&'static str, RgbaImage>,
}

impl TextureManager {
    pub fn from_assets() -> image::ImageResult<Self> {
        let mut atlas = HashMap::new();
        // === YOUR FILES (lowercase names) ===
        atlas.insert("wall_a", image::open("assets/textures/wall_a.png")?.to_rgba8());
        atlas.insert("wall_b", image::open("assets/textures/wall_b.png")?.to_rgba8());
        atlas.insert("wall_c", image::open("assets/textures/wall_c.png")?.to_rgba8());
        atlas.insert("floor",  image::open("assets/textures/floor.png")?.to_rgba8());
        atlas.insert("ceiling", image::open("assets/textures/ceiling.png")?.to_rgba8());
        Ok(Self { atlas })
    }

    #[inline]
    pub fn get_pixel_color(&self, key: &str, tx: u32, ty: u32) -> Color {
        if let Some(img) = self.atlas.get(key) {
            let (w, h) = img.dimensions();
            let x = tx.min(w.saturating_sub(1));
            let y = ty.min(h.saturating_sub(1));
            let p = img.get_pixel(x, y).0;
            return Color::new(p[0], p[1], p[2], p[3]);
        }
        Color::MAGENTA
    }

    #[inline]
    pub fn size_of(&self, key: &str) -> (u32, u32) {
        if let Some(img) = self.atlas.get(key) { img.dimensions() } else { (1, 1) }
    }
}

/// Map wall char ('#','A','B','C',...) to texture key inside the atlas.
pub fn wall_key_from_char(ch: char) -> &'static str {
    match ch {
        'A' => "wall_a",
        'B' => "wall_b",
        'C' => "wall_c",
        '#' => "wall_a", // fallback if you used '#'
        _   => "wall_a", // default
    }
}
