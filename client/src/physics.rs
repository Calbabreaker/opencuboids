use bevy_ecs::prelude::*;

pub struct Time {
    pub delta: bevy_utils::Duration,
    pub delta_seconds: f32,
    last_update: Option<bevy_utils::Instant>,
}

impl Default for Time {
    fn default() -> Time {
        Time {
            delta_seconds: 0.0,
            delta: bevy_utils::Duration::from_secs(0),
            last_update: None,
        }
    }
}

impl Time {
    pub fn update(&mut self) {
        let now = bevy_utils::Instant::now();
        if let Some(last_update) = self.last_update {
            self.delta = now - last_update;
            self.delta_seconds = self.delta.as_secs_f32();
        }

        self.last_update = Some(now);
    }

    pub fn update_system(mut res: ResMut<Self>) {
        res.update();
    }
}

#[derive(Component, Default)]
pub struct Position {
    pub vector: glam::Vec3,
}

#[derive(Component, Default)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Default)]
pub struct PhysicsBody {
    pub velocity: glam::Vec3,
    pub force: glam::Vec3,
}

pub fn physics_system(time: Res<Time>, mut query: Query<(&mut Position, &mut PhysicsBody)>) {
    for (mut position, mut body) in query.iter_mut() {
        const FRICTION: f32 = 20.0;
        let friction_force = (body.velocity * FRICTION) * time.delta_seconds;
        body.velocity = body.velocity + body.force - friction_force;
        position.vector += body.velocity * time.delta_seconds;
        body.force = glam::Vec3::ZERO;
    }
}
