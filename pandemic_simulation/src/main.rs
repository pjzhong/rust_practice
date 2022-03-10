use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_prototype_lyon::prelude::*;
use rand::Rng;

pub const FIXED_TIME_STEP: f32 = 1.0 / 60.0;

// ball params
const RADIUS: f32 = 5.0;
const NUMBALLS: i32 = 150;
const NUMBALLS_INFECTED: i32 = 10;
const MAXSPEED: f32 = 3.0;

// infection params
const INFECTION_RADIUS: f32 = 2.5;
const INFECTION_RATE: f32 = 0.50;
const BASE_RECOVERY_TIME: f32 = 10.0;
// time in seconds for recovery/death
const RECOVERY_TIME_RANGE: f32 = 1.0; // recovery_time = BASE_RECOVERY_TIME + rand(-RECOVERY_TIME_RANGE, RECOVERY_TIME_RANGE)

pub struct ColorRes {
    infected_color: DrawMode,
    normal_color: DrawMode,
    recovered_color: DrawMode,
}

#[derive(Copy, Clone, Component)]
pub struct Ball {
    speed: Vec2,
}

#[derive(Copy, Clone, Component)]
pub struct Infected {
    time_infected: f32,
}

impl Default for Infected {
    fn default() -> Self {
        Infected { time_infected: 0.0 }
    }
}

#[derive(Copy, Clone, Component)]
pub struct Recovered;

#[derive(Copy, Clone, Component)]
pub struct Health;

impl Default for Ball {
    fn default() -> Self {
        let mut speed = rand::thread_rng().gen_range(-MAXSPEED..MAXSPEED);
        if speed == 0.0 {
            speed = 1.0;
        }
        Ball {
            speed: Vec2::new(speed, speed),
        }
    }
}

impl Default for ColorRes {
    fn default() -> Self {
        ColorRes {
            infected_color: DrawMode::Outlined {
                fill_mode: FillMode::color(Color::WHITE),
                outline_mode: StrokeMode::new(Color::RED, 1.0),
            },
            normal_color: DrawMode::Outlined {
                fill_mode: FillMode::color(Color::GREEN),
                outline_mode: StrokeMode::new(Color::GREEN, 0.0),
            },
            recovered_color: DrawMode::Outlined {
                fill_mode: FillMode::color(Color::rgba(0., 82., 172., 255.)),
                outline_mode: StrokeMode::new(Color::rgba(0., 82., 172., 255.), 0.0),
            },
        }
    }
}

/// thanks https://github.com/gursi26/pandemic_simulation
fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_system(update_translation)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(game_loop_run_criteria())
                .with_system(infection_check)
                .with_system(recover_check),
        )
        .add_startup_system(setup_system)
        .run();
}

fn setup_system(mut commands: Commands, windows: Res<Windows>) {
    let colors = ColorRes::default();

    let (height, width) = if let Some(win) = windows.get_primary() {
        (win.height() / 2.0, win.width() / 2.0)
    } else {
        (100.0, 100.0)
    };

    for _ in 0..NUMBALLS {
        commands
            .spawn()
            .insert_bundle(GeometryBuilder::build_as(
                &shapes::Circle {
                    radius: 5.0,
                    center: Vec2::new(0.0, 0.0),
                },
                colors.normal_color,
                rnd_transform(height, width),
            ))
            .insert(Ball::default())
            .insert(Health);
    }

    for _ in 0..NUMBALLS_INFECTED {
        commands
            .spawn()
            .insert_bundle(GeometryBuilder::build_as(
                &shapes::Circle {
                    radius: 5.0,
                    center: Vec2::new(0.0, 0.0),
                },
                colors.infected_color,
                rnd_transform(height, width),
            ))
            .insert(Ball::default())
            .insert(Infected::default());
    }

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(ClearColor(Color::WHITE));
    commands.insert_resource(colors);
}

fn infection_check(
    mut command: Commands,
    mut normals: Query<(Entity, &Transform, &mut DrawMode), (With<Health>, Without<Recovered>)>,
    infected: Query<&Transform, With<Infected>>,
    colors: Res<ColorRes>,
) {
    let radius = Vec2::splat(RADIUS);
    let infect_radius = Vec2::splat(RADIUS + INFECTION_RADIUS);
    for infect_trans in infected.iter() {
        for (e, norm_trans, mut dm) in normals.iter_mut() {
            //先凑合用着吧
            if collide(
                infect_trans.translation,
                infect_radius,
                norm_trans.translation,
                radius,
            )
                .is_some()
            {
                let random = rand::thread_rng().gen_range(1..100) as f32 / 100.0;
                if random < INFECTION_RATE {
                    *dm = colors.infected_color;
                    command
                        .entity(e)
                        .remove::<Health>()
                        .insert(Infected::default());
                }
            }
        }
    }
}

fn recover_check(
    mut command: Commands,
    mut infected: Query<(Entity, &mut Infected, &mut DrawMode)>,
    colors: Res<ColorRes>,
) {
    for (e, mut infect, mut infect_dm) in infected.iter_mut() {
        let recovery_time = BASE_RECOVERY_TIME
            + rand::thread_rng().gen_range(-RECOVERY_TIME_RANGE..RECOVERY_TIME_RANGE);
        if infect.time_infected > recovery_time {
            *infect_dm = colors.recovered_color;
            command
                .entity(e)
                .remove::<Infected>()
                .insert_bundle((Health, Recovered));
        } else {
            infect.time_infected += FIXED_TIME_STEP;
        }
    }
}

fn update_translation(mut query: Query<(&mut Transform, &mut Ball)>, windows: Res<Windows>) {
    let (height, width) = if let Some(win) = windows.get_primary() {
        (win.height() / 2.0, win.width() / 2.0)
    } else {
        (100.0, 100.0)
    };

    for (mut trans, mut ball) in query.iter_mut() {
        trans.translation.x += ball.speed.x;
        trans.translation.y += ball.speed.y;

        if (trans.translation.x + RADIUS) > width || -width > (trans.translation.x - RADIUS) {
            ball.speed.x *= -1.0;
        }

        if (trans.translation.y + RADIUS) > height || -height > (trans.translation.y - RADIUS) {
            ball.speed.y *= -1.0;
        }
    }
}

fn rnd_transform(height: f32, width: f32) -> Transform {
    let mut rng = rand::thread_rng();
    let width = width - RADIUS;
    let height = height - RADIUS;
    let posx = rng.gen_range(-width..width);
    let posy = rng.gen_range(-height..height);

    Transform {
        translation: Vec2::new(posx, posy).extend(0.0),
        ..Default::default()
    }
}

pub fn game_loop_run_criteria() -> FixedTimestep {
    FixedTimestep::step(FIXED_TIME_STEP as f64).with_label("fixed_update_run_criteria")
}
