use glam::{Quat, Vec3};

#[derive(Debug)]
pub struct Camera {
    pub location: Vec3,
    pub rotation: Quat,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            location: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        }
    }
}

#[derive(Debug)]
pub struct ArcballRig {
    pub distance: f32,
    pub focus: Vec3,
    pub rotation: Quat,
}

impl ArcballRig {
    pub fn new() -> Self {
        let distance = 50.0;
        let focus = Vec3::ZERO;
        let rotation = Quat::IDENTITY;
        Self {
            distance,
            focus,
            rotation,
        }
    }

    pub fn mouse_motion(&mut self, delta: (f64, f64)) {
        // log::debug!(
        //     "camera.pos {:?}",
        //     self.rotation * Vec3::X * self.distance + self.focus
        // );

        // These work individually
        let rx = Quat::from_axis_angle(Vec3::Y, delta.0 as f32 * 0.001);
        let ry = Quat::from_axis_angle(self.rotation * Vec3::X, -delta.1 as f32 * 0.001);
        // Doesn't combine well
        let r = rx * ry;

        self.rotation = (r * self.rotation).normalize();
    }

    pub fn mouse_wheel(&mut self, delta: (f32, f32)) {
        self.distance = (self.distance - delta.1 * 10.0).max(0.0);
    }
}
