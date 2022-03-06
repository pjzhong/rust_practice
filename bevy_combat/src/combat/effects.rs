use bevy::prelude::*;

use super::{attack::FireInfo, Target};

/// Transform of the effect source.
pub struct SourceTransform(pub GlobalTransform);

/// The location where an effect is applied.
pub struct EffectLocation(pub Vec3);

/// The entity responsible for causing an effect.
pub struct Instigator(pub Entity);

/// The effectiveness of an effect. Effects start with an effectiveness of 1.0
pub struct Effectiveness(pub f32);

impl Default for Effectiveness {
    fn default() -> Self {
        Effectiveness(1.0)
    }
}

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum EffectSystems {
    RemoveOldEffects,
    ApplyEffects,
}

pub struct Effect;

type Spawner = fn(&mut Commands) -> Entity;

pub struct Effector {
    pub spawn_effect: Spawner,
}

pub fn apply_effects(
    mut commands: Commands,
    mut query: Query<(Entity, &Target, &GlobalTransform, &mut FireInfo, &Effector)>,
    pos_query: Query<&GlobalTransform>,
) {
    for (entity, target, transform, mut weapon, effect) in query.iter_mut() {
        if !weapon.firing {
            continue;
        }

        if let Some(target) = target.0 {
            weapon.firing = false;

            let spawned = (effect.spawn_effect)(&mut commands);
            commands.entity(spawned).insert_bundle((
                Target(Some(target)),
                Instigator(entity),
                SourceTransform(*transform),
                Effectiveness::default(),
                Effect,
            ));

            if let Ok(target_transform) = pos_query.get_component::<GlobalTransform>(target) {
                commands.entity(spawned).insert(EffectLocation(target_transform.translation));
            }
        }
    }
}

/// Deletes old effect entities.
pub fn remove_old_effects(mut commands: Commands, query: Query<Entity, With<Effect>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
