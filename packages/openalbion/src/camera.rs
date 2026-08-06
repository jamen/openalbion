//! Fly camera.
//!
//! World space is **Z-up**, matching the game: `BuildOuterSkyMesh` puts the dome apex at
//! Z = 7000 and `VSHADER_LANDSCAPE_FOREGROUND` assembles position as (v0.x, v0.y, v1.x)
//! with height third. Every mesh, def value and data-layer coordinate stays in that
//! convention; the single conversion into the renderer's -Z-forward/+Y-up view space
//! happens here, in `view_matrix`. See AGENTS.md §3.6.

use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
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
    /// Rotation about world +Z, radians. 0 looks down +X.
    yaw: f32,
    /// Elevation above the XY plane, radians.
    pitch: f32,
}

impl Camera {
    /// World up. Z-up, per AGENTS.md §3.6.
    pub const UP: Vec3 = Vec3::Z;

    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            fov_y: 70.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            fly_speed: 10.0,
            mouse_delta: (0.0, 0.0),
            mouse_sensitivity: 0.003,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn set_aspect(&mut self, width: u32, height: u32) {
        if height > 0 {
            self.aspect = width as f32 / height as f32;
        }
    }

    /// Unit view direction in world (Z-up) space.
    pub fn forward(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        Vec3::new(cos_pitch * cos_yaw, cos_pitch * sin_yaw, sin_pitch)
    }

    /// Unit rightward vector, perpendicular to `forward` and world up.
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Self::UP).normalize_or(Vec3::Y)
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward(), Self::UP)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// View-projection with the camera translation removed, so the sky dome renders
    /// centred on the viewer with no parallax.
    pub fn sky_view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * Mat4::look_to_rh(Vec3::ZERO, self.forward(), Self::UP)
    }

    /// Aim at `target` from the current position.
    pub fn look_at(&mut self, target: Vec3) {
        let d = target - self.position;
        let horizontal = d.truncate().length();
        if horizontal > f32::EPSILON || d.z.abs() > f32::EPSILON {
            self.yaw = d.y.atan2(d.x);
            self.pitch = d.z.atan2(horizontal);
        }
    }

    /// Accumulate mouse delta for look input.
    pub fn process_mouse(&mut self, dx: f32, dy: f32) {
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
    }

    /// Apply fly movement. `keys` is (forward, backward, left, right, up, down).
    pub fn fly(&mut self, dt: f32, keys: (bool, bool, bool, bool, bool, bool), speed_mult: f32) {
        self.yaw -= self.mouse_delta.0 * self.mouse_sensitivity;
        self.pitch -= self.mouse_delta.1 * self.mouse_sensitivity;
        self.pitch = self
            .pitch
            .clamp(-85.0_f32.to_radians(), 85.0_f32.to_radians());
        self.mouse_delta = (0.0, 0.0);

        let forward = self.forward();
        let right = self.right();

        let mut velocity = Vec3::ZERO;
        if keys.0 {
            velocity += forward;
        }
        if keys.1 {
            velocity -= forward;
        }
        if keys.2 {
            velocity -= right;
        }
        if keys.3 {
            velocity += right;
        }
        if keys.4 {
            velocity += Self::UP;
        }
        if keys.5 {
            velocity -= Self::UP;
        }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize() * self.fly_speed * speed_mult;
        }

        self.position += velocity * dt;
    }

    /// Update near/far planes based on world extents.
    pub fn set_world_extents(&mut self, world_span: f32) {
        self.near = world_span * 0.0001;
        self.far = world_span * 10.0; // covers the full world plus sky
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
