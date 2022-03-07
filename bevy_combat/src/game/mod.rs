use crate::constants::FIXED_TIME_STEP;
use bevy::app::{AppBuilder, CoreStage, Events, Plugin};
use bevy::core::FixedTimestep;
use bevy::prelude::{Commands, IntoSystem, Res, SystemStage};
use bevy::window::{WindowId, WindowResized, Windows};

pub fn game_loop_run_criteria() -> FixedTimestep {
    FixedTimestep::step(FIXED_TIME_STEP as f64).with_label("fixed_update_run_criteria")
}

pub struct GameTimeDelta(pub f32);

pub struct GameSpeed(pub i32);

pub struct GameConfig {
    pub max_x: f32,
    pub min_x: f32,
    pub max_y: f32,
    pub min_y: f32,
}

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
        app.add_system(resize_notificator.system())
            .add_startup_system(startup.system());
    }
}

fn startup(mut commands: Commands, windows: Res<Windows>) {
    commands.insert_resource(GameTimeDelta(1.0 / 60.0));
    commands.insert_resource(GameSpeed(2));

    if let Some(e) = windows.get_primary() {
        let max_x = e.width() / 2.0;
        let min_x = -max_x;
        let max_y = e.height() / 2.0;
        let min_y = -max_y;
        commands.remove_resource::<GameConfig>();
        commands.insert_resource(GameConfig {
            max_x,
            min_x,
            max_y,
            min_y,
        });
    }
}

fn resize_notificator(mut commands: Commands, resize_event: Res<Events<WindowResized>>) {
    let mut reader = resize_event.get_reader();
    for e in reader.iter(&resize_event) {
        if e.id == WindowId::primary() {
            let max_x = e.width / 2.0;
            let min_x = -max_x;
            let max_y = e.height / 2.0;
            let min_y = -max_y;
            commands.remove_resource::<GameConfig>();
            commands.insert_resource(GameConfig {
                max_x,
                min_x,
                max_y,
                min_y,
            });
        }
    }
}
