use bevy::app::App;
use bevy::asset::{AssetServer, Assets};
use bevy::prelude::*;
use bevy::prelude::{ClearColor, Color, Commands, Res, ResMut};
use bevy::sprite::ColorMaterial;
use bevy::DefaultPlugins;
use bevy_combat::ai::aggression::{AggroRadius, RetargetBehavior};
use bevy_combat::ai::idle::IdleBehavior;
use bevy_combat::ai::movement::TurnToDestinationBehavior;
use bevy_combat::ai::AIPlugin;
use bevy_combat::combat::attack::{CoolDown, FireInfo};
use bevy_combat::combat::effects::Effector;
use bevy_combat::combat::mortal::{Health, MaxHealth, Mortal};
use bevy_combat::combat::{CombatPlugin, Target, Team};
use bevy_combat::fx::animated::AnimatedEffects;
use bevy_combat::fx::death::DeathEffect;
use bevy_combat::fx::EffectsPlugin;
use bevy_combat::game::BaseGamePlugin;
use bevy_combat::movement::{Mass, MaxTurnSpeed, MovementBundle, MovementPlugin, Thrust};
use bevy_combat::templates::weapons::{pulse_laser_attack, small_pulse_laser_attack};
use rand::Rng;

/// Thanks ['bevy_combat']
///
/// ['bevy_combat']: https://github.com/ElliotB256/bevy_combat
fn main() {
    let mut app = App::build();

    app.add_plugins(DefaultPlugins)
        .add_plugin(BaseGamePlugin)
        .add_plugin(AIPlugin)
        .add_plugin(MovementPlugin)
        .add_plugin(EffectsPlugin)
        .add_plugin(CombatPlugin)
        .add_startup_system(setup.system());

    app.run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();

    let tile_size = Vec2::splat(16.0);

    let sprite_handle = materials.add(assets.load("art/smallship.png").into());
    let drones = materials.add(assets.load("art/drone.png").into());

    commands.insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.8)));

    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());

    for _ in 0..20 {
        let position =
            Vec2::new(-20.0, 0.0) + Vec2::new(rng.gen_range(-5.0..5.0), rng.gen_range(-20.0..20.0));
        let translation = (position * tile_size).extend(0.0);
        let rotation = Quat::from_rotation_z(rng.gen::<f32>());
        let scale = Vec3::splat(1.0);

        commands
            .spawn()
            .insert_bundle(SpriteBundle {
                material: sprite_handle.clone(),
                transform: Transform {
                    translation,
                    rotation,
                    scale,
                },
                sprite: Sprite::new(tile_size),
                ..Default::default()
            })
            .insert_bundle(MovementBundle {
                max_turn_speed: MaxTurnSpeed::new(3.0),
                mass: Mass(1.0),
                thrust: Thrust(150.0),
                ..Default::default()
            })
            .insert(IdleBehavior)
            .insert(TurnToDestinationBehavior {
                destination: Vec3::default(),
            })
            .insert_bundle((
                AggroRadius(1000.0),
                Target::default(),
                Team(1),
                RetargetBehavior {
                    interval: 4.0,
                    remaining_time: 4.0,
                },
                Health(100.0),
                MaxHealth(100.0),
                Mortal,
            ))
            .insert_bundle((
                CoolDown::new(1.0),
                FireInfo {
                    range: 100.0,
                    cone: 0.15,
                    firing: false,
                },
                Effector {
                    spawn_effect: pulse_laser_attack,
                },
            ))
            .insert(DeathEffect {
                time_to_explosion: 0.1,
                time_to_smoke: 0.05,
                dying_explosion: AnimatedEffects::SmallExplosion,
                death_explosion: AnimatedEffects::MediumExplosion,
            });
    }

    for _ in 0..60 {
        let drone_size = Vec2::splat(8.0);
        let position =
            Vec2::new(60.0, 0.0) + Vec2::new(rng.gen_range(-5.0..5.0), rng.gen_range(-30.0..30.0));
        let translation = (position * drone_size).extend(0.0);
        let rotation = Quat::from_rotation_z(rng.gen::<f32>());
        let scale = Vec3::splat(1.0);

        commands
            .spawn()
            .insert_bundle(SpriteBundle {
                material: drones.clone(),
                transform: Transform {
                    translation,
                    rotation,
                    scale,
                },
                sprite: Sprite::new(drone_size),
                ..Default::default()
            })
            .insert_bundle(MovementBundle {
                max_turn_speed: MaxTurnSpeed::new(4.0),
                mass: Mass(1.0),
                thrust: Thrust(250.0),
                ..Default::default()
            })
            .insert(IdleBehavior)
            .insert(TurnToDestinationBehavior {
                destination: Vec3::default(),
            })
            .insert_bundle((
                AggroRadius(1000.0),
                Target::default(),
                Team(2),
                RetargetBehavior {
                    interval: 4.0,
                    remaining_time: 4.0,
                },
                Health(50.0),
                MaxHealth(50.0),
                Mortal,
            ))
            .insert_bundle((
                CoolDown::new(0.2),
                FireInfo {
                    range: 80.0,
                    cone: 0.3,
                    firing: false,
                },
                Effector {
                    spawn_effect: small_pulse_laser_attack,
                },
            ))
            .insert(DeathEffect {
                time_to_explosion: 0.1,
                time_to_smoke: 0.05,
                dying_explosion: AnimatedEffects::SmallExplosion,
                death_explosion: AnimatedEffects::MediumExplosion,
            });
    }
}
