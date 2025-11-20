use raylib::math::Vector3;

/// Input del jugador para la nave
#[derive(Debug, Clone, Copy)]
pub struct ShipInput {
    pub forward: f32,   // W/S: -1.0 a 1.0
    pub lateral: f32,   // A/D: -1.0 a 1.0
    pub vertical: f32,  // Space/Ctrl: -1.0 a 1.0
    pub rotation: f32,  // Q/E: -1.0 a 1.0 (yaw)
    pub nitro: bool,    // Shift
}

/// Estado de la cámara (yaw, pitch, distancia)
pub struct CameraState {
    pub yaw: f32,      // Radianes
    pub pitch: f32,    // Radianes
    pub distance: f32, // Distancia de la cámara a la nave
}

impl CameraState {
    pub fn new(yaw: f32, pitch: f32, distance: f32) -> Self {
        Self { yaw, pitch, distance }
    }

    /// Calcula el vector forward de la cámara
    pub fn forward(&self) -> Vector3 {
        let (sin_y, cos_y) = self.yaw.sin_cos();
        let (sin_p, cos_p) = self.pitch.sin_cos();

        Vector3 {
            x: sin_y * cos_p,
            y: -sin_p,
            z: cos_y * cos_p,
        }
    }

    /// Calcula el vector right de la cámara
    pub fn right(&self) -> Vector3 {
        let (sin_y, cos_y) = self.yaw.sin_cos();
        Vector3 {
            x: cos_y,
            y: 0.0,
            z: -sin_y,
        }
    }
}

/// Nave espacial con física simple
pub struct Ship {
    pub position: Vector3,
    pub velocity: Vector3,
    
    // Parámetros de física
    pub thrust: f32,
    pub nitro_thrust: f32,
    pub friction: f32,
    pub rotation_speed: f32,
}

impl Ship {
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            velocity: Vector3::zero(),
            thrust: 0.3,
            nitro_thrust: 0.6,
            friction: 0.88,
            rotation_speed: 2.0, // rad/s
        }
    }

    /// Aplica input del jugador y actualiza física
    pub fn apply_input(&mut self, dt: f32, input: ShipInput, cam_state: &mut CameraState) {
        // Actualizar rotación de cámara (yaw)
        cam_state.yaw += input.rotation * self.rotation_speed * dt;

        // Thrust actual (nitro o normal)
        let current_thrust = if input.nitro {
            self.nitro_thrust
        } else {
            self.thrust
        };

        // Calcular vectores de dirección basados en la cámara
        let forward = cam_state.forward();
        let right = cam_state.right();
        let up = Vector3::up();

        // Aplicar aceleración en base a input
        let accel = Vector3 {
            x: forward.x * input.forward + right.x * input.lateral,
            y: up.y * input.vertical,
            z: forward.z * input.forward + right.z * input.lateral,
        };

        // Actualizar velocidad
        self.velocity.x += accel.x * current_thrust;
        self.velocity.y += accel.y * current_thrust;
        self.velocity.z += accel.z * current_thrust;

        // Aplicar fricción
        let friction_factor = self.friction.powf(dt * 60.0);
        self.velocity.x *= friction_factor;
        self.velocity.y *= friction_factor;
        self.velocity.z *= friction_factor;

        // Integrar posición
        self.position.x += self.velocity.x * dt * 60.0;
        self.position.y += self.velocity.y * dt * 60.0;
        self.position.z += self.velocity.z * dt * 60.0;
    }

    /// Resetea velocidad (útil para colisiones)
    pub fn stop(&mut self) {
        self.velocity = Vector3::zero();
    }

    /// Resetea posición y orientación
    pub fn reset(&mut self, position: Vector3, cam_state: &mut CameraState) {
        self.position = position;
        self.velocity = Vector3::zero();
        cam_state.yaw = 0.0;
        cam_state.pitch = 0.0;
    }
}
