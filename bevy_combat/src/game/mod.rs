use crate::constants::FIXED_TIME_STEP;
use bevy::app::{AppBuilder, CoreStage, Plugin};
use bevy::core::FixedTimestep;
use bevy::prelude::{Commands, IntoSystem, SystemStage};

pub fn game_loop_run_criteria() -> FixedTimestep {
    FixedTimestep::step(FIXED_TIME_STEP as f64).with_label("fixed_update_run_criteria")
}

pub struct GameTimeDelta(pub f32);

pub struct GameSpeed(pub i32);

pub static DESPAWN_STAGE: &str = "despawn_stage";

#[derive(Default)]
pub struct BaseGamePlugin;

impl Plugin for BaseGamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_stage_after(
            CoreStage::Update,
            DESPAWN_STAGE,
            SystemStage::single_threaded(),
        );
        app.add_startup_system(startup.system());
    }
}

fn startup(mut commands: Commands) {
    commands.insert_resource(GameTimeDelta(1.0 / 60.0));
    commands.insert_resource(GameSpeed(2));
}
