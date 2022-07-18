use bevy_ecs::event::Events;
use bevy_ecs::prelude::*;
use camera::Camera;
use input::Input;
use physics::{movement, PhysicsBody, Position, Rotation, Time};
use player::{camera_update, player_movement, Player};
use renderer::{render, viewport_resize, RendererData, ViewportResizeEvent};
use state::State;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod camera;
mod input;
mod physics;
mod player;
mod renderer;
mod state;
mod texture;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let state = pollster::block_on(State::new(&event_loop));

    let mut world = World::new();
    world.insert_resource(RendererData::new(&state));
    world.insert_resource(Input::default());
    world.insert_resource(Time::default());
    world.insert_resource(state);
    world.insert_resource(Camera::new(f32::to_radians(60.0), 0.01, 1000.0));
    world.insert_resource(Events::<ViewportResizeEvent>::default());

    world
        .spawn()
        .insert(Position {
            vector: glam::vec3(0.0, 0.0, -2.0),
        })
        .insert(PhysicsBody::default())
        .insert(Rotation { x: 90.0, y: 0.0 })
        .insert(Player);

    let mut schedule = Schedule::default();
    schedule.add_stage(
        "update",
        SystemStage::single_threaded()
            .with_system(Events::<ViewportResizeEvent>::update_system)
            .with_system(Time::update_system)
            .with_system(Input::update_system)
            .with_system(player_movement)
            .with_system(movement),
    );
    schedule.add_stage(
        "prerender",
        SystemStage::single_threaded()
            .with_system(viewport_resize)
            .with_system(camera_update),
    );
    schedule.add_stage("render", SystemStage::parallel().with_system(render));

    event_loop.run(move |event, _, control_flow| {
        let mut viewport_resize_event = world
            .get_resource_mut::<Events<ViewportResizeEvent>>()
            .unwrap();

        match event {
            Event::MainEventsCleared => {
                schedule.run(&mut world);
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    viewport_resize_event.send(ViewportResizeEvent(*size));
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    viewport_resize_event.send(ViewportResizeEvent(**new_inner_size));
                }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => world
                .get_resource_mut::<Input>()
                .unwrap()
                .process_event(&event),
            _ => (),
        }
    });
}
