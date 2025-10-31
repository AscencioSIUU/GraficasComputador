use raylib::prelude::*;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Color>,
    pub zbuffer: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer: vec![Color::BLACK; width * height],
            zbuffer: vec![f32::INFINITY; width * height],
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.buffer.fill(color);
        self.zbuffer.fill(f32::INFINITY);
    }

    pub fn point(&mut self, x: i32, y: i32, color: Color, depth: f32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let index = y as usize * self.width + x as usize;
        
        if depth < self.zbuffer[index] {
            self.buffer[index] = color;
            self.zbuffer[index] = depth;
        }
    }

    pub fn to_image(&self) -> Image {
        let image = Image::gen_image_color(self.width as i32, self.height as i32, Color::BLACK);
        
        unsafe {
            let image_data = image.data as *mut Color;
            for i in 0..self.buffer.len() {
                *image_data.add(i) = self.buffer[i];
            }
        }
        
        image
    }
}
