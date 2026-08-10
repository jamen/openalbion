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
    /// **Horizontal** field of view, radians. `CCamera` (`bbblibrary/lib_camera.hpp:1005`)
    /// carries `HorizontalFOV` with the vertical derived; `camera_mode.def` gives every
    /// `CAMERA_MODE` an `FOV`, and `CAMERA_MODE_TEMPLATE` defaults to 70. AGENTS.md §3.10.
    pub fov_h: f32,
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
            fov_h: 70.0_f32.to_radians(),
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

    /// Vertical field of view in radians, derived from the horizontal FOV and the aspect
    /// ratio exactly as the engine does.
    ///
    /// `CEngineCamera` builds the projection (`fableengine/engine_camera.cpp:781-793`) as:
    ///
    /// ```text
    /// AspectRatio          = width / height
    /// HomogenousViewScaleX = 1 / tan(HFOV/2)
    /// HomogenousViewScaleY = HomogenousViewScaleX * AspectRatio   // Use2DFOV == false
    /// ```
    ///
    /// then scales view-space X by `ScaleX` and Y by `ScaleY`. A standard right-handed
    /// perspective has `m00 = f/aspect`, `m11 = f` with `f = 1/tan(fovY/2)` — the same
    /// `m11 = m00 * aspect` relation — so equating the two gives
    ///
    /// ```text
    /// fovY = 2 * atan( tan(HFOV/2) / aspect )
    /// ```
    ///
    /// At the 70° template default and 16:9 that is ~43°, *not* 70°. AGENTS.md §3.10.
    ///
    /// `Use2DFOV` selects a second path where the vertical FOV is independent
    /// (`ENGINE.FOV_2D`); that is the 2D/UI camera and does not apply here.
    pub fn fov_y(&self) -> f32 {
        2.0 * ((self.fov_h * 0.5).tan() / self.aspect).atan()
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y(), self.aspect, self.near, self.far)
    }

    /// `c5..c8` for geometry that arrives in **absolute world coordinates** — the static
    /// mesh pass. `VSHADER_STATIC_DIRLIGHT` transforms straight from the vertex
    /// (`dp4 oPos, v0, c5..c8`) with no `-c4`, because the object matrix
    /// (`CalcObjectMatrix`) has already placed the mesh in the world.
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// `c5..c8` for geometry that arrives **relative to the camera** — the sky and the
    /// landscape.
    ///
    /// The three world shaders only agree on one reading of `c5..c8`: it carries a
    /// rotation-only view. `VSHADER_OUTER_SKY` transforms a dome built around the origin
    /// with no translation term at all, so the dome can only end up around the viewer if
    /// the view matrix has none; `VSHADER_LANDSCAPE_FOREGROUND` then supplies the
    /// translation itself with `add r1, r0, -c4` (`c4` = `CameraPos`).
    ///
    /// Feeding [`Self::view_projection_matrix`] to a pass that also subtracts the camera
    /// position displaces its geometry by `-camera_pos`, which is what the landscape was
    /// doing.
    pub fn camera_relative_view_projection_matrix(&self) -> Mat4 {
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

#[cfg(test)]
mod tests {
    use super::Camera;

    /// The engine derives the vertical FOV from the horizontal one and the aspect ratio
    /// (`engine_camera.cpp:781-793`). At 70° horizontal and 16:9 that is ~43° vertical —
    /// feeding the def's number straight in as a vertical FOV gives a far too wide view.
    #[test]
    fn vertical_fov_is_derived_from_horizontal_and_aspect() {
        let mut camera = Camera::new();
        camera.set_aspect(1280, 720);

        assert!((camera.aspect - 16.0 / 9.0).abs() < 1e-6);
        assert!(
            (camera.fov_y().to_degrees() - 42.99).abs() < 0.05,
            "got {}",
            camera.fov_y().to_degrees()
        );

        // A square viewport makes horizontal and vertical coincide.
        camera.set_aspect(720, 720);
        assert!((camera.fov_y().to_degrees() - 70.0).abs() < 0.01);
    }

    /// The two `c5..c8` matrices must agree: a world point transformed absolutely (the
    /// static mesh path) and the same point transformed camera-relative (the landscape and
    /// sky path) have to land on the same pixel, or models float free of the ground.
    #[test]
    fn absolute_and_camera_relative_transforms_agree() {
        let mut camera = Camera::new();
        camera.set_aspect(1280, 720);
        camera.position = glam::Vec3::new(100.0, 60.0, 40.0);
        camera.look_at(glam::Vec3::new(100.0, 120.0, 22.0));

        let point = glam::Vec3::new(103.0, 118.0, 25.0);

        let ndc = |v: glam::Vec4| v.truncate() / v.w;
        let absolute = ndc(camera.view_projection_matrix() * point.extend(1.0));
        let relative = ndc(
            camera.camera_relative_view_projection_matrix()
                * (point - camera.position).extend(1.0),
        );

        assert!(
            (absolute - relative).length() < 1e-5,
            "absolute {absolute:?} vs camera-relative {relative:?}"
        );
    }
}
