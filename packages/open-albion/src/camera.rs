use glam::{Mat4, Quat, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub orientation: Quat,
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    /// Current fly speed (world units per second).
    pub fly_speed: f32,
    /// Accumulated mouse delta for look input.
    mouse_delta: (f32, f32),
    /// Mouse sensitivity (radians per pixel).
    mouse_sensitivity: f32,
    /// Pitch (X-axis rotation) in radians.
    pitch: f32,
    /// Yaw (Y-axis rotation) in radians.
    yaw: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            fov_y: 70.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            fly_speed: 10.0,
            mouse_delta: (0.0, 0.0),
            mouse_sensitivity: 0.003,
            pitch: 0.0,
            yaw: 0.0,
        }
    }

    pub fn set_aspect(&mut self, width: u32, height: u32) {
        if height > 0 {
            self.aspect = width as f32 / height as f32;
        }
    }

    pub fn forward(&self) -> Vec3 {
        self.orientation * -Vec3::Z
    }

    pub fn right(&self) -> Vec3 {
        self.orientation * Vec3::X
    }

    pub fn view_matrix(&self) -> Mat4 {
        let inv_orientation = self.orientation.conjugate();
        Mat4::from_quat(inv_orientation) * Mat4::from_translation(-self.position)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn sky_view_projection_matrix(&self) -> Mat4 {
        let forward = self.forward();
        let horiz = glam::Vec3::new(forward.x, 0.0, forward.z);
        let len = horiz.length();
        if len < 0.0001 {
            return self.projection_matrix();
        }
        let horiz = horiz / len;
        let right = glam::Vec3::new(-horiz.z, 0.0, horiz.x);
        let rotation = Mat4::from_cols(
            right.extend(0.0),
            glam::Vec3::Y.extend(0.0),
            (-horiz).extend(0.0),
            glam::Vec3::ZERO.extend(1.0),
        );
        self.projection_matrix() * rotation
    }

    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.position).normalize();
        let right = forward.cross(up).normalize();
        let corrected_up = right.cross(forward);

        let rotation_matrix = Mat4::from_cols(
            right.extend(0.0),
            corrected_up.extend(0.0),
            (-forward).extend(0.0),
            Vec3::ZERO.extend(1.0),
        );

        self.orientation = Quat::from_mat4(&rotation_matrix);
    }

    /// Apply accumulated mouse delta as yaw/pitch rotation.
    pub fn process_mouse(&mut self, dx: f32, dy: f32) {
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
    }

    /// Rebuild orientation from yaw + pitch.
    fn update_orientation(&mut self) {
        let pitch_clamped = self.pitch.clamp(
            -85.0_f32.to_radians(),
            85.0_f32.to_radians(),
        );
        let yaw_quat = Quat::from_rotation_y(self.yaw);
        let pitch_quat = Quat::from_rotation_x(pitch_clamped);
        self.orientation = yaw_quat * pitch_quat;
    }

    /// Apply fly movement. `keys` is (forward, backward, left, right, up, down).
    pub fn fly(
        &mut self,
        dt: f32,
        keys: (bool, bool, bool, bool, bool, bool),
        speed_mult: f32,
    ) {
        self.yaw -= self.mouse_delta.0 * self.mouse_sensitivity;
        self.pitch -= self.mouse_delta.1 * self.mouse_sensitivity;
        self.mouse_delta = (0.0, 0.0);
        self.update_orientation();

        let speed = self.fly_speed * speed_mult;
        let forward = self.forward();
        let right = self.right();

        let mut velocity = Vec3::ZERO;
        if keys.0 { velocity += forward; }
        if keys.1 { velocity -= forward; }
        if keys.2 { velocity -= right; }
        if keys.3 { velocity += right; }
        if keys.4 { velocity += Vec3::Y; }
        if keys.5 { velocity -= Vec3::Y; }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize() * speed;
        }

        self.position += velocity * dt;
    }

    /// Update near/far planes based on world extents.
    pub fn set_world_extents(&mut self, world_span: f32) {
        self.near = world_span * 0.0001; // ~0.01 for 100-unit spans
        self.far = world_span * 10.0; // covers the full world plus sky
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
