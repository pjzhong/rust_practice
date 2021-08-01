use bevy::core::FixedTimestep;
use bevy::prelude::*;

use crate::{
    FromPlayer, Laser, Materials, Player, PlayerReaderFire, PlayerState, Speed, WinSize,
    PLAYER_RESPAWN_DELAY, SCALE, TIME_STEP,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(PlayerState::default())
            .add_startup_stage(
                "game_set_actors",
                SystemStage::single(player_spawn.system()),
            )
            .add_system(player_movement.system())
            .add_system(player_fire.system())
            .add_system(laser_movement.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.5))
                    .with_system(player_spawn.system()),
            );
    }
}

fn player_spawn(
    mut commands: Commands,
    materials: Res<Materials>,
    win_size: Res<WinSize>,
    time: Res<Time>,
    mut player_state: ResMut<PlayerState>,
) {
    let now = time.seconds_since_startup();
    let last_shot = player_state.last_short;

    //spawn a sprite
    if !player_state.on && (last_shot == 0. || now > last_shot + PLAYER_RESPAWN_DELAY) {
        let bottom = -win_size.h / 2.;
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.player.clone(),
                transform: Transform {
                    translation: Vec3::new(0., bottom + 30., 10.),
                    scale: Vec3::new(SCALE, SCALE, 1.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Player)
            .insert(PlayerReaderFire(true))
            .insert(Speed::default());

        player_state.spawned();
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    win_size: Res<WinSize>,
    mut query: Query<(&Speed, &mut Transform, With<Player>)>,
) {
    if let Ok((speed, mut transform, _)) = query.single_mut() {
        let limit = win_size.w / 2.;
        let dir = if keyboard_input.pressed(KeyCode::Left) && -limit < transform.translation.x {
            -1.
        } else if keyboard_input.pressed(KeyCode::Right) && transform.translation.x < limit {
            1.
        } else {
            0.
        };

        transform.translation.x += dir * speed.0 * TIME_STEP;
    }
}

fn player_fire(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    materials: Res<Materials>,
    mut query: Query<(&Transform, &mut PlayerReaderFire, With<Player>)>,
) {
    if let Ok((player_tf, mut read_fire, _)) = query.single_mut() {
        if read_fire.0 && kb.pressed(KeyCode::Space) {
            let x = player_tf.translation.x;
            let y = player_tf.translation.y;

            let mut spawn_lasers = |x_offset: f32| {
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.player_laser.clone(),
                        transform: Transform {
                            translation: Vec3::new(x + x_offset, y + 15., 0.),
                            scale: Vec3::new(SCALE, SCALE, 1.),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Laser)
                    .insert(FromPlayer)
                    .insert(Speed::default());
            };

            let x_offset = 144.0 / 4.0 - 5.0;
            spawn_lasers(x_offset);
            spawn_lasers(-x_offset);

            read_fire.0 = false;
        }

        if kb.just_released(KeyCode::Space) {
            read_fire.0 = true;
        }
    }
}

fn laser_movement(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(
        Entity,
        &Speed,
        &mut Transform,
        (With<Laser>, With<FromPlayer>),
    )>,
) {
    for (laser_entity, speed, mut laser_tf, _) in query.iter_mut() {
        let translation = &mut laser_tf.translation;
        translation.y += speed.0 * TIME_STEP;
        if translation.y > win_size.h {
            commands.entity(laser_entity).despawn();
        }
    }
}
