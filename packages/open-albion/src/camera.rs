use glam::{Mat4, Quat, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub orientation: Quat,
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            fov_y: std::f32::consts::FRAC_PI_4,
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
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

    pub fn up(&self) -> Vec3 {
        self.orientation * Vec3::Y
    }

    pub fn view_matrix(&self) -> Mat4 {
        let inv_orientation = self.orientation.conjugate();
        let rotated_position = inv_orientation * -self.position;
        Mat4::from_quat(inv_orientation) * Mat4::from_translation(rotated_position)
    }

    pub fn view_matrix_rotation_only(&self) -> Mat4 {
        Mat4::from_quat(self.orientation.conjugate())
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn sky_view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix_rotation_only()
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
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AnimatedCamera {
    pub camera: Camera,
    pub time: f32,
}

impl AnimatedCamera {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
            time: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;

        let t = self.time * 0.3;

        let radius = 0.5;
        let x = radius * t.sin();
        let z = radius * (t * 2.0).sin() * 0.5;
        let y = 0.0;

        self.camera.position = Vec3::new(x, y, z);

        let yaw = t * 0.5;
        let pitch = (t * 0.7).sin() * 0.3;

        let yaw_quat = Quat::from_rotation_y(yaw);
        let pitch_quat = Quat::from_rotation_x(pitch);

        self.camera.orientation = yaw_quat * pitch_quat;
    }

    pub fn sky_view_projection(&self) -> [[f32; 4]; 4] {
        self.camera.sky_view_projection_matrix().to_cols_array_2d()
    }
}

impl Default for AnimatedCamera {
    fn default() -> Self {
        Self::new()
    }
}
