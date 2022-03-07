use bevy::prelude::*;

use crate::combat::damage::Damage;
use crate::{
    combat::attack::Attack,
    fx::{beams::BeamStyle, HitEffect},
};

/// TODO Damage
/// The attack from a small pulsed laser.
pub fn pulse_laser_attack(commands: &mut Commands) -> Entity {
    commands
        .spawn()
        .insert_bundle((
            Attack::new(3.0),
            Damage(20.0),
            BeamStyle {
                effect: crate::fx::animated::AnimatedEffects::BlueLaserBeam,
                width: 1.0,
            },
            HitEffect {
                effect: crate::fx::animated::AnimatedEffects::SmallExplosion,
            },
        ))
        .id()
}

pub fn small_pulse_laser_attack(commands: &mut Commands) -> Entity {
    commands
        .spawn()
        .insert_bundle((
            Attack::new(2.0),
            Damage(2.0),
            BeamStyle {
                effect: crate::fx::animated::AnimatedEffects::GreenLaserBeam,
                width: 0.5,
            },
            HitEffect {
                effect: crate::fx::animated::AnimatedEffects::TinyPlusExplosion,
            },
        ))
        .id()
}
