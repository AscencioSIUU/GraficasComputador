//! Sistema de carga de texturas para el raytracer optimizado.

use crate::lighting::Tex;
use image::GenericImageView;

/// Estructura que almacena los buffers de todas las texturas cargadas.
pub struct TextureStorage {
    pub grass_cover_buf: Vec<u8>,
    pub grass_cover_wh: (u32, u32),
    
    pub grass_side_buf: Vec<u8>,
    pub grass_side_wh: (u32, u32),
    
    pub dirt_buf: Vec<u8>,
    pub dirt_wh: (u32, u32),
    
    pub stone_buf: Vec<u8>,
    pub stone_wh: (u32, u32),
    
    pub wood_buf: Vec<u8>,
    pub wood_wh: (u32, u32),
    
    pub leaves_buf: Vec<u8>,
    pub leaves_wh: (u32, u32),
    
    pub water_buf: Vec<u8>,
    pub water_wh: (u32, u32),
    
    pub lava_buf: Vec<u8>,
    pub lava_wh: (u32, u32),
    
    pub obsidian_buf: Vec<u8>,
    pub obsidian_wh: (u32, u32),
    
    pub glowstone_buf: Vec<u8>,
    pub glowstone_wh: (u32, u32),
    
    pub diamond_buf: Vec<u8>,
    pub diamond_wh: (u32, u32),
    
    pub iron_buf: Vec<u8>,
    pub iron_wh: (u32, u32),
    
    pub chest_buf: Vec<u8>,
    pub chest_wh: (u32, u32),
    
    pub ice_buf: Vec<u8>,
    pub ice_wh: (u32, u32),
    
    pub portal_buf: Vec<u8>,
    pub portal_wh: (u32, u32),
    
    pub torch_buf: Vec<u8>,
    pub torch_wh: (u32, u32),
    
    pub clouds_buf: Vec<u8>,
    pub clouds_wh: (u32, u32),
}

impl TextureStorage {
    /// Carga todas las texturas desde el directorio assets/
    pub fn load() -> Self {
        println!("Cargando texturas PNG para raytracing...");
        
        let grass_cover = load_or_default("assets/grass_top_16x16.png", (128, 128, 70));
        let grass_side = load_or_default("assets/grass_side_16x16.png", (128, 128, 70));
        let dirt = load_or_default("assets/dirt_16x16.png", (134, 96, 67));
        let stone = load_or_default("assets/stone_16x16.png", (128, 128, 128));
        let wood = load_or_default("assets/wood_16x16.png", (139, 90, 43));
        let leaves = load_or_default("assets/leaves_16x16.png", (80, 160, 80));
        let water = load_or_default("assets/water.png", (50, 100, 200));
        let lava = load_or_default("assets/lava.png", (255, 100, 0));
        let obsidian = load_or_default("assets/obsidian_16x16.png", (20, 10, 30));
        let glowstone = load_or_default("assets/glowstone.png", (255, 230, 180));
        let diamond = load_or_default("assets/diamond.png", (150, 230, 255));
        let iron = load_or_default("assets/iron.png", (180, 180, 180));
        let chest = load_or_default("assets/chest.png", (160, 100, 50));
        let ice = load_or_default("assets/ice_16x16.png", (180, 200, 255));
        let portal = load_or_default("assets/portal.png", (150, 50, 255));
        let torch = load_or_default("assets/torch_16x16.png", (255, 180, 80));
        let clouds = load_or_default("assets/clouds.png", (200, 220, 255));
        
        Self {
            grass_cover_buf: grass_cover.0,
            grass_cover_wh: grass_cover.1,
            
            grass_side_buf: grass_side.0,
            grass_side_wh: grass_side.1,
            
            dirt_buf: dirt.0,
            dirt_wh: dirt.1,
            
            stone_buf: stone.0,
            stone_wh: stone.1,
            
            wood_buf: wood.0,
            wood_wh: wood.1,
            
            leaves_buf: leaves.0,
            leaves_wh: leaves.1,
            
            water_buf: water.0,
            water_wh: water.1,
            
            lava_buf: lava.0,
            lava_wh: lava.1,
            
            obsidian_buf: obsidian.0,
            obsidian_wh: obsidian.1,
            
            glowstone_buf: glowstone.0,
            glowstone_wh: glowstone.1,
            
            diamond_buf: diamond.0,
            diamond_wh: diamond.1,
            
            iron_buf: iron.0,
            iron_wh: iron.1,
            
            chest_buf: chest.0,
            chest_wh: chest.1,
            
            ice_buf: ice.0,
            ice_wh: ice.1,
            
            portal_buf: portal.0,
            portal_wh: portal.1,
            
            torch_buf: torch.0,
            torch_wh: torch.1,
            
            clouds_buf: clouds.0,
            clouds_wh: clouds.1,
        }
    }
    
    /// Obtiene la textura de grass_cover como Tex
    pub fn get_grass_cover(&self) -> Tex {
        Tex {
            pix: &self.grass_cover_buf,
            w: self.grass_cover_wh.0,
            h: self.grass_cover_wh.1,
        }
    }
    
    pub fn get_grass_side(&self) -> Tex {
        Tex {
            pix: &self.grass_side_buf,
            w: self.grass_side_wh.0,
            h: self.grass_side_wh.1,
        }
    }
    
    pub fn get_dirt(&self) -> Tex {
        Tex {
            pix: &self.dirt_buf,
            w: self.dirt_wh.0,
            h: self.dirt_wh.1,
        }
    }
    
    pub fn get_stone(&self) -> Tex {
        Tex {
            pix: &self.stone_buf,
            w: self.stone_wh.0,
            h: self.stone_wh.1,
        }
    }
    
    pub fn get_wood(&self) -> Tex {
        Tex {
            pix: &self.wood_buf,
            w: self.wood_wh.0,
            h: self.wood_wh.1,
        }
    }
    
    pub fn get_leaves(&self) -> Tex {
        Tex {
            pix: &self.leaves_buf,
            w: self.leaves_wh.0,
            h: self.leaves_wh.1,
        }
    }
    
    pub fn get_water(&self) -> Tex {
        Tex {
            pix: &self.water_buf,
            w: self.water_wh.0,
            h: self.water_wh.1,
        }
    }
    
    pub fn get_lava(&self) -> Tex {
        Tex {
            pix: &self.lava_buf,
            w: self.lava_wh.0,
            h: self.lava_wh.1,
        }
    }
    
    pub fn get_obsidian(&self) -> Tex {
        Tex {
            pix: &self.obsidian_buf,
            w: self.obsidian_wh.0,
            h: self.obsidian_wh.1,
        }
    }
    
    pub fn get_glowstone(&self) -> Tex {
        Tex {
            pix: &self.glowstone_buf,
            w: self.glowstone_wh.0,
            h: self.glowstone_wh.1,
        }
    }
    
    pub fn get_diamond(&self) -> Tex {
        Tex {
            pix: &self.diamond_buf,
            w: self.diamond_wh.0,
            h: self.diamond_wh.1,
        }
    }
    
    pub fn get_iron(&self) -> Tex {
        Tex {
            pix: &self.iron_buf,
            w: self.iron_wh.0,
            h: self.iron_wh.1,
        }
    }
    
    pub fn get_chest(&self) -> Tex {
        Tex {
            pix: &self.chest_buf,
            w: self.chest_wh.0,
            h: self.chest_wh.1,
        }
    }
    
    pub fn get_ice(&self) -> Tex {
        Tex {
            pix: &self.ice_buf,
            w: self.ice_wh.0,
            h: self.ice_wh.1,
        }
    }
    
    pub fn get_portal(&self) -> Tex {
        Tex {
            pix: &self.portal_buf,
            w: self.portal_wh.0,
            h: self.portal_wh.1,
        }
    }
    
    pub fn get_torch(&self) -> Tex {
        Tex {
            pix: &self.torch_buf,
            w: self.torch_wh.0,
            h: self.torch_wh.1,
        }
    }
    
    pub fn get_clouds(&self) -> Tex {
        Tex {
            pix: &self.clouds_buf,
            w: self.clouds_wh.0,
            h: self.clouds_wh.1,
        }
    }
}

/// Carga una textura o genera una de color sólido si falla.
fn load_or_default(path: &str, fallback_rgb: (u8, u8, u8)) -> (Vec<u8>, (u32, u32)) {
    match image::open(path) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            println!("  ✓ Cargada: {} ({}x{})", path, w, h);
            (rgba.into_raw(), (w, h))
        }
        Err(_) => {
            println!("  ✗ No encontrada: {} (usando color sólido)", path);
            let w = 16u32;
            let h = 16u32;
            let mut buf = Vec::with_capacity((w * h * 4) as usize);
            for _ in 0..(w * h) {
                buf.push(fallback_rgb.0);
                buf.push(fallback_rgb.1);
                buf.push(fallback_rgb.2);
                buf.push(255);
            }
            (buf, (w, h))
        }
    }
}
