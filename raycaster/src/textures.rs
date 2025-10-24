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
        atlas.insert("wall_d", image::open("assets/textures/wall_d.png")?.to_rgba8());
        atlas.insert("wall_e", image::open("assets/textures/wall_e.png")?.to_rgba8());
        atlas.insert("wall_f", image::open("assets/textures/wall_f.png")?.to_rgba8());
        atlas.insert("floor",  image::open("assets/textures/floor.png")?.to_rgba8());
        atlas.insert("ceiling", image::open("assets/textures/ceiling.png")?.to_rgba8());
        atlas.insert("coin", image::open("assets/textures/coin.png")?.to_rgba8());
        atlas.insert("enemy", image::open("assets/textures/enemy.png")?.to_rgba8());
        atlas.insert("pistol_view", image::open("assets/textures/pistol_view.png")?.to_rgba8());
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

    /// Sample texture using usize coordinates (useful where texture sizes are usize).
    #[inline]
    pub fn sample_at(&self, key: &str, tx: usize, ty: usize) -> Color {
        if let Some(img) = self.atlas.get(key) {
            let (w, h) = img.dimensions();
            let x = (tx as u32).min(w.saturating_sub(1));
            let y = (ty as u32).min(h.saturating_sub(1));
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
    match ch.to_ascii_uppercase() {
        'A' => "wall_a",
        'B' => "wall_b",
        'C' => "wall_c",
        'D' => "wall_d",
        'E' => "wall_e",
        'F' => "wall_f",
        '#' => "wall_a", // fallback if you used '#'
        _   => "wall_a", // default
    }
}

/// Choose a wall texture key based on map char and cell coordinates.
/// This produces variety: A and D are predominant; B/C/E/F appear as alternates.
pub fn wall_key_from_char_at(ch: char, cell_x: i32, cell_y: i32) -> &'static str {
    // If map explicitly encodes a specific wall, keep it
    match ch.to_ascii_uppercase() {
        'A' | 'B' | 'C' | 'D' | 'E' | 'F' => return wall_key_from_char(ch),
        _ => {}
    }

    // Otherwise pick based on coordinates to mix textures.
    let mut idx = (cell_x.wrapping_abs() as usize).wrapping_add(cell_y.wrapping_abs() as usize);
    // spread pattern a bit
    idx = idx.wrapping_mul(31).wrapping_add(7);
    if idx % 5 < 3 {
        // predominant group (A or D)
        if idx % 2 == 0 { "wall_a" } else { "wall_d" }
    } else {
        // alternating group among B,C,E,F
        match idx % 4 {
            0 => "wall_b",
            1 => "wall_c",
            2 => "wall_e",
            _ => "wall_f",
        }
    }
}
