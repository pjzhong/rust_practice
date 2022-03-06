pub mod attack;
pub mod effects;

use crate::combat::attack::{fire, update_cool_downs, AttackSystems};
use crate::combat::effects::{apply_effects, remove_old_effects, EffectSystems};
use crate::game::game_loop_run_criteria;
use bevy::app::{AppBuilder, CoreStage, Plugin};
use bevy::prelude::*;

#[derive(Clone, Copy, Default)]
pub struct Target(pub Option<Entity>);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Team(pub i32);

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum CombatSystems {
    Set,
}

#[derive(Default)]
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set_to_stage(
            CoreStage::Update,
            SystemSet::new()
                .label(CombatSystems::Set)
                .with_run_criteria(game_loop_run_criteria())
                .with_system(
                    update_cool_downs
                        .system()
                        .label(AttackSystems::UpdateCoolDowns),
                )
                .with_system(fire.system().label(AttackSystems::FireAttack))
                .with_system(apply_effects.system().label(EffectSystems::ApplyEffects))
                .with_system(
                    remove_old_effects
                        .system()
                        .label(EffectSystems::RemoveOldEffects),
                ),
        );
    }
}
