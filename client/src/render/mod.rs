mod bind_group;
mod buffer;
mod chunk_renderer;
mod main_renderer;
mod render_pipeline;
mod texture;

use self::{
    chunk_renderer::{chunk_mesh_gen, chunk_render, ChunkRenderer},
    main_renderer::{on_resize, post_render, pre_render, MainRenderer, RenderState},
};
use crate::window::Window;
use bevy_ecs::prelude::*;

#[derive(Default)]
pub struct Plugin;

#[derive(SystemLabel, Clone, Copy, Hash, Debug, Eq, PartialEq)]
struct RenderPass;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        let render_stage = SystemStage::parallel()
            .with_system(on_resize.before(pre_render))
            .with_system(chunk_mesh_gen)
            .with_system(pre_render.before(RenderPass))
            .with_system(chunk_render.label(RenderPass))
            .with_system(post_render.after(RenderPass));

        let window = app.world.resource::<Window>();
        app.insert_resource(pollster::block_on(RenderState::new(window)))
            .init_resource::<ChunkRenderer>()
            .init_resource::<MainRenderer>()
            .add_stage_after(bevy_app::CoreStage::PostUpdate, "render", render_stage);
    }
}
