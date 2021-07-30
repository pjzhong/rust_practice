use bevy::prelude::*;

const PLAYER_SPRITE: &str = "player_a_01.png";

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Rust Invaders!".to_string(),
            width: 598.0,
            height: 676.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    assert_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // postion windows
    let mut windows = match windows.get_primary_mut() {
        None => {
            //TODO tell me what wrong
            return;
        }
        Some(w) => w,
    };

    //spawn a sprite
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(assert_server.load(PLAYER_SPRITE).into()),
        ..Default::default()
    });
}
