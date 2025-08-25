use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
}
impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    pub fn clear(&mut self) {}
    pub fn set_pixel(&mut self, _x: u32, _y: u32) {}
    pub fn set_current_color(&mut self, _c: Color) {}
    pub fn swap_buffers(&self, _rl: &mut RaylibHandle, _th: &RaylibThread) {}
}
