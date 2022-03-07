use crate::combat::attack::{Attack, AttackResult};
use crate::combat::effects::Effect;
use crate::combat::mortal::Health;
use crate::combat::Target;
use bevy::prelude::*;

/// Entity will deal a specified amount of damage.
pub struct Damage(pub f32);

pub fn apply_damage(
    query: Query<(&Target, &Damage, &Attack), With<Effect>>,
    mut health_query: Query<&mut Health>,
) {
    for (target, damage, attack) in query.iter() {
        if attack.result != AttackResult::Hit {
            continue;
        }

        if let Some(target) = target.0 {
            if let Ok(mut health) = health_query.get_mut(target) {
                health.0 -= damage.0;
            }
        }
    }
}

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum DamageSystems {
    ApplyDamage,
}
