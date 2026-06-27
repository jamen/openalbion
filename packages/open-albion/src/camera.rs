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
        // Fable's dome is built with Z=zenith, X/Y=horizon ring. Our dome uses
        // Y=zenith, X/Z=horizon ring. Both need the dome zenith locked to screen-top.
        //
        // Use only the camera's horizontal direction (yaw) so the zenith always
        // points to the top of the screen. Ignore pitch to avoid the degenerate
        // case where the camera looks nearly straight down.
        let forward = self.forward();
        let horiz = glam::Vec3::new(forward.x, 0.0, forward.z);
        let len = horiz.length();
        if len < 0.0001 {
            // Looking straight up or down — identity rotation so dome covers the view.
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
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
