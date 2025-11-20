use raylib::prelude::*;

pub struct ShipControls {
    pub position: Vector3,
    pub velocity: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl ShipControls {
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            velocity: Vector3::zero(),
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        }
    }

    pub fn forward(&self) -> Vector3 {
        let cy = self.yaw.cos();
        let sy = self.yaw.sin();
        let cp = self.pitch.cos();
        let sp = self.pitch.sin();
        Vector3::new(sy * cp, sp, cy * cp).normalized()
    }

    pub fn right(&self) -> Vector3 {
        let f = self.forward();
        Vector3::new(f.z, 0.0, -f.x).normalized()
    }

    pub fn up(&self) -> Vector3 {
        let f = self.forward();
        let r = self.right();
        r.cross(f).normalized()
    }

    pub fn update(&mut self, rl: &RaylibHandle, _mouse_sensitivity: f32, move_speed: f32) {
        self.apply_input(rl, move_speed);
        self.position = self.position + self.velocity;
        self.velocity = self.velocity * 0.9;
    }

    fn apply_input(&mut self, rl: &RaylibHandle, move_speed: f32) {
        // W/S for Z axis movement, A/D for X axis movement
        if rl.is_key_down(KeyboardKey::KEY_W) {
            self.velocity.z += move_speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_S) {
            self.velocity.z -= move_speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_A) {
            self.velocity.x -= move_speed;
        }
        if rl.is_key_down(KeyboardKey::KEY_D) {
            self.velocity.x += move_speed;
        }
    }
}
