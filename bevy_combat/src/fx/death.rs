use crate::combat::mortal::Dieing;
use crate::fx::animated::{AnimatedEffects, CreateAnimatedEffect};
use crate::game::GameTimeDelta;
use bevy::prelude::{Commands, GlobalTransform, Query, Res, Transform, Vec3};
use rand::Rng;

/// Generates effects while the entity is dieing.
pub struct DeathEffect {
    pub time_to_explosion: f32,
    pub time_to_smoke: f32,
    pub dying_explosion: AnimatedEffects,
    pub death_explosion: AnimatedEffects,
}

pub fn do_death_effects(
    mut commands: Commands,
    dt: Res<GameTimeDelta>,
    mut query: Query<(&mut DeathEffect, &GlobalTransform, &Dieing)>,
) {
    let mut rng = rand::thread_rng();
    for (mut death_effect, transform, dieing) in query.iter_mut() {
        death_effect.time_to_explosion -= dt.0;
        death_effect.time_to_smoke -= dt.0;

        if death_effect.time_to_explosion < 0.0 {
            let x_offset: f32 = rng.gen_range(-6.0..6.0);
            let y_offset: f32 = rng.gen_range(-6.0..6.0);
            death_effect.time_to_explosion = rng.gen_range(0.05..0.2);
            commands.spawn().insert(CreateAnimatedEffect {
                effect: death_effect.dying_explosion,
                transform: Transform::from_translation(
                    transform.translation + Vec3::new(x_offset, y_offset, 0.1),
                ),
                parent: None,
            });
        }

        if death_effect.time_to_smoke < 0.0 {
            let x_offset: f32 = rng.gen_range(-6.0..6.0);
            let y_offset: f32 = rng.gen_range(-6.0..6.0);
            death_effect.time_to_smoke = rng.gen_range(0.0..0.05);
            commands.spawn().insert(CreateAnimatedEffect {
                effect: AnimatedEffects::Smoke,
                transform: Transform::from_translation(
                    transform.translation + Vec3::new(x_offset, y_offset, -0.05),
                ),
                parent: None,
            });
        }

        if dieing.dead {
            commands.spawn().insert(CreateAnimatedEffect {
                effect: death_effect.death_explosion,
                transform: Transform::from_translation(transform.translation),
                parent: None,
            });
        }
    }
}
