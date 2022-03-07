use bevy::prelude::*;
use rand::Rng;

use crate::combat::attack::AttackResult;
use crate::combat::{
    attack::Attack,
    effects::{EffectLocation, Instigator, SourceTransform},
    CombatSystems, Target,
};
use crate::fx::HitEffect;

use super::animated::{AnimatedEffects, CreateAnimatedEffect};

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum BeamSystems {
    Set,
}

pub struct BeamStyle {
    pub effect: AnimatedEffects,
    pub width: f32,
}

#[derive(Clone, Copy)]
pub struct BeamTracking {
    pub target: Entity,
    pub source: Entity,
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    pub track_target: bool,
}

pub struct BeamEffectPlugin;

impl Plugin for BeamEffectPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new()
                .label(BeamSystems::Set)
                .after(CombatSystems::Set)
                .with_system(spawn_hit_effects.system())
                .with_system(beams_tracking.system())
                .with_system(spawn_beams.system()),
        );
    }
}

fn get_transform(start: Vec3, end: Vec3, width: f32) -> Transform {
    let delta = end - start;
    let angle = delta.y.atan2(delta.x) + std::f32::consts::FRAC_PI_2;
    let scale = Vec3::new(width, delta.length() / 4.0, 1.0);

    Transform::from_translation((end + start) / 2.0)
        * Transform::from_rotation(Quat::from_rotation_z(angle))
        * Transform::from_scale(scale)
}

fn spawn_beams(
    mut commands: Commands,
    query: Query<(
        &BeamStyle,
        &SourceTransform,
        &EffectLocation,
        &Target,
        &Instigator,
        &Attack,
    )>,
) {
    for (beam_style, source, effect, target, instigator, attack) in query.iter() {
        if let Some(target) = target.0 {
            let transform = get_transform(source.0.translation, effect.0, beam_style.width);
            commands
                .spawn()
                .insert(CreateAnimatedEffect {
                    transform,
                    effect: beam_style.effect,
                    parent: None,
                })
                .insert(BeamTracking {
                    target,
                    source: instigator.0,
                    start: source.0.translation,
                    end: effect.0,
                    width: beam_style.width,
                    track_target: attack.result == AttackResult::Hit,
                });
        }
    }
}

fn beams_tracking(
    mut query: Query<(&mut BeamTracking, &mut Transform)>,
    world_query: Query<&GlobalTransform>,
) {
    for (mut tracking, mut transform) in query.iter_mut() {
        if let Ok(start_t) = world_query.get_component::<GlobalTransform>(tracking.source) {
            tracking.start = start_t.translation;
        }

        if tracking.track_target {
            if let Ok(end_t) = world_query.get_component::<GlobalTransform>(tracking.target) {
                tracking.end = end_t.translation;
            }
        }

        *transform = get_transform(tracking.start, tracking.end, tracking.width);
    }
}

fn spawn_hit_effects(mut commands: Commands, query: Query<(&HitEffect, &EffectLocation, &Attack)>) {
    let mut rng = rand::thread_rng();

    for (hit_effect, location, attack) in query.iter() {
        if attack.result == AttackResult::Miss {
            continue;
        }

        let x_offset: f32 = rng.gen_range(-6.0..6.0);
        let y_offset: f32 = rng.gen_range(-6.0..6.0);
        commands.spawn().insert(CreateAnimatedEffect {
            effect: hit_effect.effect,
            transform: Transform::from_translation(location.0 + Vec3::new(x_offset, y_offset, 0.1)),
            parent: None,
        });
    }
}
