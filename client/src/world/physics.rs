use bevy_ecs::prelude::*;

use crate::time::Time;

#[derive(Component, Default)]
pub struct WorldPosition(pub glam::Vec3);

#[derive(Component, Default)]
pub struct WorldRotation {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Default)]
pub struct PhysicsBody {
    pub velocity: glam::Vec3,
    pub force: glam::Vec3,
}

pub fn physics(time: Res<Time>, mut query: Query<(&mut WorldPosition, &mut PhysicsBody)>) {
    for (mut position, mut body) in query.iter_mut() {
        const FRICTION: f32 = 20.0;
        let friction_force = (body.velocity * FRICTION) * time.delta_seconds;
        body.velocity = body.velocity + body.force - friction_force;
        position.0 += body.velocity * time.delta_seconds;
        body.force = glam::Vec3::ZERO;
    }
}
