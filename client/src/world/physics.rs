use bevy_ecs::prelude::*;

use crate::time::Time;

#[derive(Component, Clone, Copy, Default)]
pub struct WorldTransform {
    pub position: glam::Vec3,
    pub rotation: glam::Vec2,
}

#[derive(Component, Default)]
pub struct PhysicsBody {
    pub velocity: glam::Vec3,
    pub force: glam::Vec3,
}

pub fn physics(time: Res<Time>, mut query: Query<(&mut WorldTransform, &mut PhysicsBody)>) {
    const FRICTION: f32 = 20.0;

    let delta = time.delta.as_secs_f32();
    for (mut transform, mut body) in query.iter_mut() {
        let friction_force = body.velocity * f32::min(FRICTION * delta, 1.0);
        body.velocity = body.velocity + body.force - friction_force;
        transform.position += body.velocity * delta;
        body.force = glam::Vec3::ZERO;
    }
}
