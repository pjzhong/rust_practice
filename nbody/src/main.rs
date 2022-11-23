use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
    hash::Hash,
};

use bevy::{prelude::*, reflect::erased_serde::__private::serde::__private::de};
use bevy_prototype_lyon::{
    prelude::{DrawMode, FillMode, GeometryBuilder, ShapePlugin, StrokeMode},
    shapes,
};
use derive_more::Deref;
use rand::Rng;

//Events
struct Reset;

#[derive(Component, Debug, Clone)]
struct Planet {
    radius: f32,
    density: f32,
    color: Color,
    is_sun: bool,
}

impl Planet {
    pub fn mass(&self) -> f32 {
        self.density * (4.0 / 3.0) * PI * self.radius.powf(3.0)
    }
}

#[derive(Component, Debug, Clone, Deref)]
struct Velocity(Vec2);

#[derive(Clone)]
struct Settings {
    n_objects: usize,
    collisions: bool,
    min_planet_size: f32,
    max_planet_size: f32,
    min_planet_density: f32,
    max_planet_density: f32,
    min_planet_orbit_radius: f32,
    max_planet_orbit_radius: f32,
    sun_size: f32,
    sun_density: f32,
    g: f32,
    time_step: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            n_objects: 500,
            collisions: true,
            min_planet_size: 0.5,
            max_planet_size: 3.5,
            min_planet_density: 0.5,
            max_planet_density: 2.0,
            min_planet_orbit_radius: 100.0,
            max_planet_orbit_radius: 1000.0,
            sun_size: 30.0,
            sun_density: 5.0,
            g: 3.5,
            time_step: 120.0,
        }
    }
}

pub fn game() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Settings::default())
        .add_event::<Reset>()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup)
        .add_system(setup_many_orbits)
        .add_system(gravity)
        .run();
}

fn setup(mut commands: Commands, mut ev_reset: EventWriter<Reset>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    ev_reset.send(Reset);
}

fn gravity(
    mut commands: Commands,
    mut planet_query: Query<(Entity, &mut Planet, &mut Velocity, &mut Transform)>,
    settings: Res<Settings>,
) {
    let mut accel_map: HashMap<u32, Vec2> = HashMap::new();
    let mut despawned: HashSet<u32> = HashSet::new();

    for (ent_1, planet_1, vel_1, trans_1) in planet_query.iter() {
     
        let id1 = ent_1.id();
        let mut accel_cum = Vec2::new(0.0, 0.0);
        for (ent_2, planet_2, vel_2, trans_2) in planet_query.iter() {
            if despawned.contains(&id1) {
                break;
            }
            let id2 = ent_2.id();
            if id1 == id2 ||  despawned.contains(&id2){
                continue;
            }

            let r_vector = trans_1.translation.truncate() - trans_2.translation.truncate();
            if settings.collisions && r_vector.length() < planet_1.radius + planet_2.radius {
                let sum_mass = planet_1.mass() + planet_2.mass();
                let final_velocity = Velocity(
                    vel_1.0 * planet_1.mass() / sum_mass + vel_2.0 * planet_2.mass() / sum_mass,
                );
                commands.entity(ent_2).despawn();
                commands.entity(ent_1).despawn();
                despawned.insert(id2);
                despawned.insert(id1);

                if planet_1.mass() > planet_2.mass() {
                    spawn_planet(
                        &mut commands,
                        merge_planets(planet_1, planet_2),
                        final_velocity,
                        *trans_1,
                    );
                } else {
                    spawn_planet(
                        &mut commands,
                        merge_planets(planet_2, planet_1),
                        final_velocity,
                        *trans_2,
                    );
                }
            } else {
                let r_mag = r_vector.length();
                let r_mag = if !settings.collisions && r_mag < planet_1.radius + planet_2.radius {
                    planet_1.radius + planet_2.radius
                } else {
                    r_mag
                };

                let accel = -1.0 * settings.g * planet_2.mass() / r_mag.powf(2.0);
                let r_vector_unit = r_vector / r_mag;
                accel_cum += accel * r_vector_unit;
            }
        }

        accel_map.insert(id1, accel_cum);
    }

    for (entity, _, mut vel, mut trans) in planet_query.iter_mut() {
        let id = entity.id();
        if despawned.contains(&id) {
            continue;
        }

        if let Some(acc) = accel_map.get(&id) {
            vel.0 += *acc * (1.0 / settings.time_step);
            trans.translation.x += vel.x * (1.0 / settings.time_step);
            trans.translation.y += vel.y * (1.0 / settings.time_step);
        }
    }
}

fn setup_many_orbits(
    plaent_query: Query<(Entity, &Planet)>,
    mut ev_reset: EventReader<Reset>,
    mut commands: Commands,
    settings: Res<Settings>,
) {
    let manual_reset = ev_reset.iter().next().is_some();
    if manual_reset {
        for (ent, _) in plaent_query.iter() {
            commands.entity(ent).despawn();
        }

        let mut rng = rand::thread_rng();
        let sun = Planet {
            radius: settings.sun_size,
            density: settings.sun_density,
            color: Color::YELLOW,
            is_sun: true,
        };
        spawn_planet(
            &mut commands,
            sun.clone(),
            Velocity(Vec2::new(0.0, 0.0)),
            Transform::from_xyz(0.0, 0.0, 0.0),
        );

        let radius_rang = settings.max_planet_size - settings.min_planet_size;
        let density_rang = settings.max_planet_density - settings.min_planet_density;
        let orbit_range = settings.max_planet_orbit_radius - settings.min_planet_orbit_radius;
        for _ in 0..settings.n_objects {
            let radius = rng.gen::<f32>() * radius_rang + settings.min_planet_size;
            let density = rng.gen::<f32>() * density_rang + settings.min_planet_density;
            let planet = Planet {
                radius,
                density,
                color: Color::WHITE,
                is_sun: false,
            };

            let orbit_radius: f32 =
                rng.gen::<f32>() * orbit_range + settings.min_planet_orbit_radius;
            let radian = rng.gen::<f32>() * 2.0 * PI;
            let (x, y) = (orbit_radius * radian.cos(), orbit_radius * radian.sin());
            let orbital_velocity = (settings.g * sun.mass() / orbit_radius).sqrt();
            let (vx, vy) = (
                -orbital_velocity * radian.sin(),
                orbital_velocity * radian.cos(),
            );
            spawn_planet(
                &mut commands,
                planet,
                Velocity(Vec2::new(vx, vy)),
                Transform::from_xyz(x, y, 10.0),
            );
        }
    }
}

fn spawn_planet(commands: &mut Commands, planet: Planet, velocity: Velocity, transform: Transform) {
    let shape = shapes::Circle {
        radius: planet.radius,
        center: Default::default(),
    };

    let mut entity_commands = commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Outlined {
            fill_mode: FillMode::color(planet.color),
            outline_mode: StrokeMode::new(planet.color, 0.0),
        },
        transform,
    ));
    entity_commands.insert(planet).insert(velocity);
}

fn radius_to_area(r: f32) -> f32 {
    PI * r.powf(2.0)
}

fn radius_to_volume(r: f32) -> f32 {
    4.0 / 3.0 * PI * r.powf(3.0)
}

fn area_to_radius(a: f32) -> f32 {
    (a / PI).sqrt()
}

fn volume_to_radius(v: f32) -> f32 {
    ((3.0 * v) / (4.0 * PI)).powf(1.0 / 3.0)
}

fn merge_planets(planet_1: &Planet, planet_2: &Planet) -> Planet {
    let volume_1 = radius_to_volume(planet_1.radius);
    let volume_2 = radius_to_volume(planet_2.radius);
    let volume_sum = volume_1 + volume_2;
    let new_radius = volume_to_radius(volume_sum);
    Planet {
        radius: new_radius,
        density: planet_1.density * (volume_1 / volume_sum)
            + planet_2.density * (volume_2 / volume_sum),
        color: planet_1.color,
        is_sun: planet_1.is_sun || planet_2.is_sun,
    }
}

fn main() {
    game();
}
