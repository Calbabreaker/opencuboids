use bevy_ecs::prelude::*;
use renderer::{render, RendererData};
use state::State;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

mod renderer;
mod state;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut world = World::new();

    let state = pollster::block_on(State::new(&event_loop));
    world.insert_resource(RendererData::new(&state));
    world.insert_resource(state);

    let mut schedule = Schedule::default();
    schedule.add_stage("render", SystemStage::parallel().with_system(render));

    event_loop.run(move |event, _, control_flow| {
        let mut state = world.get_resource_mut::<State>().unwrap();

        match event {
            Event::MainEventsCleared => {
                schedule.run(&mut world);
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    state.resize(*size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                _ => (),
            },
            _ => (),
        }
    });
}
