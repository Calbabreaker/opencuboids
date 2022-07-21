use crate::world::{WorldPosition, WorldRotation};

pub struct Camera {
    view_projection: glam::Mat4,
    pub fov_radians: f32,
    pub near: f32,
    pub far: f32,
    aspect_ratio: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view_projection: glam::Mat4::IDENTITY,
            fov_radians: f32::to_radians(60.0),
            near: 0.01,
            far: 1000.0,
            aspect_ratio: 0.0,
        }
    }
}

impl Camera {
    // Updates the view projection matrix for later use for rendering
    pub fn update(&mut self, position: &WorldPosition, rotation: &WorldRotation) {
        let yaw = rotation.x.to_radians();
        let pitch = rotation.y.to_radians();
        let direction = glam::vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        );
        let front = direction.normalize();

        let view = glam::Mat4::look_at_lh(position.0, position.0 + front, glam::Vec3::Y);
        self.view_projection = self.get_projection() * view;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect_ratio = (width as f32) / (height as f32);
    }

    pub fn get_view_projection(&self) -> glam::Mat4 {
        self.view_projection
    }

    pub fn get_projection(&self) -> glam::Mat4 {
        glam::Mat4::perspective_lh(self.fov_radians, self.aspect_ratio, self.near, self.far)
    }
}
