use raylib::prelude::*;

pub struct Framebuffer {
    pub color_buffer: Image,
    width: i32,
    height: i32,
    foreground_color: Color,
    background_color: Color,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width, height, background_color);
        Self {
            width,
            height,
            color_buffer,
            background_color,
            foreground_color: Color::WHITE,
        }
    }

    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(self.width, self.height, self.background_color);
        // Alternatively, no re-alloc:
        // self.color_buffer.clear_background(self.background_color);
    }

    pub fn set_foreground_color(&mut self, color: Color) {
        self.foreground_color = color;
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_pixel(&mut self, x: i32, y: i32) {
        self.color_buffer.draw_pixel(x, y, self.foreground_color);
    }

    // Optional helpers (may be unused)
    pub fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        self.color_buffer.draw_pixel(x, y, color);
    }

    pub fn line(&mut self, xi: i32, yi: i32, xf: i32, yf: i32) {
        self.color_buffer.draw_line(xi, yi, xf, yf, self.foreground_color);
    }

    pub fn render_to_png(&self, filename: &str) {
        self.color_buffer.export_image(filename);
    }
}
