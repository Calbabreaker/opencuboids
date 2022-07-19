use crate::physics::{Position, Rotation};

pub struct Camera {
    view_projection: glam::Mat4,
    pub fov_radians: f32,
    pub near: f32,
    pub far: f32,
    aspect_ratio: f32,
}

impl Camera {
    pub fn new(fov_radians: f32, near: f32, far: f32) -> Self {
        Self {
            view_projection: glam::Mat4::IDENTITY,
            fov_radians,
            near,
            far,
            aspect_ratio: 0.0,
        }
    }

    // Updates the view projection matrix for later use for rendering
    pub fn update(&mut self, position: &Position, rotation: &Rotation) {
        let yaw = rotation.x.to_radians();
        let pitch = rotation.y.to_radians();
        let direction = glam::vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        );
        let front = direction.normalize();

        let view = glam::Mat4::look_at_lh(position.vector, position.vector + front, glam::Vec3::Y);
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
