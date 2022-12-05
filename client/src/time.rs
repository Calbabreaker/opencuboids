use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct Time {
    pub delta: bevy_utils::Duration,
    last_update: bevy_utils::Instant,
    pub frame_rate: u32,
    frame_rate_counter: u32,
    last_frame_rate_show: bevy_utils::Instant,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta: bevy_utils::Duration::default(),
            last_update: bevy_utils::Instant::now(),
            frame_rate: 0,
            frame_rate_counter: 0,
            last_frame_rate_show: bevy_utils::Instant::now(),
        }
    }
}

impl Time {
    pub fn update(&mut self) {
        let now = bevy_utils::Instant::now();
        self.delta = now - self.last_update;

        self.frame_rate_counter += 1;
        if now - self.last_frame_rate_show > bevy_utils::Duration::from_secs(1) {
            self.frame_rate = self.frame_rate_counter;
            self.last_frame_rate_show = now;
            self.frame_rate_counter = 0;
            log::info!("Frame rate: {}", self.frame_rate);
        }

        self.last_update = now;
    }

    pub fn update_system(mut res: ResMut<Self>) {
        res.update();
    }
}

#[derive(Default)]
pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_resource::<Time>()
            .add_system_to_stage(bevy_app::CoreStage::PreUpdate, Time::update_system);
    }
}
