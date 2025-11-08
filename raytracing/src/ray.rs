//! Rayo con origen y dirección.

use crate::math::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub orig: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(orig: Vec3, dir: Vec3) -> Self {
        Self { orig, dir }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.orig.add(self.dir.mul(t))
    }
}
