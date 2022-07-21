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

#[derive(Default)]
pub struct TimePlugin;

impl bevy_app::Plugin for TimePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<Time>()
            .add_system_to_stage(bevy_app::CoreStage::PreUpdate, Time::update_system);
    }
}
