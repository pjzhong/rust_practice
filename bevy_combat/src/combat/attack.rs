use crate::combat::Target;
use crate::game::GameTimeDelta;
use bevy::prelude::*;

#[derive(PartialEq, Clone, Copy)]
pub enum AttackResult {
    Hit,
    Miss,
    Blocked,
}

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum AttackSystems {
    FireAttack,
    UpdateCoolDowns,
}

pub struct FireInfo {
    /// The targeting angular cone of the tool in radians.
    pub cone: f32,
    /// The range at which the tools effect can be applied.
    pub range: f32,
    /// True if the tool is currently being fired this instant.
    pub firing: bool,
}

/// Cooldown timer for a Weapon.
pub struct CoolDown {
    /// Time remaining on the cooldown.
    pub remaining: f32,
    /// Total duration of the cooldown.
    pub duration: f32,
}

impl CoolDown {
    pub fn reset(&mut self) {
        self.remaining = self.duration;
    }
    /// Is the cooldown ready?
    pub fn is_ready(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Is in cooldown
    pub fn not_ready(&self) -> bool {
        0.0 < self.remaining
    }

    pub fn new(duration: f32) -> Self {
        CoolDown {
            remaining: duration,
            duration,
        }
    }
}

pub struct Attack {
    pub accuracy: f32,
    pub result: AttackResult,
}

impl Attack {
    pub fn new(accuracy: f32) -> Self {
        Attack {
            accuracy,
            result: AttackResult::Hit,
        }
    }
}

pub fn update_cool_downs(dt: Res<GameTimeDelta>, mut query: Query<&mut CoolDown>) {
    for mut cool_down in query.iter_mut() {
        cool_down.remaining -= dt.0;
    }
}

pub fn fire(
    mut query: Query<(&mut CoolDown, &mut FireInfo, &Target, &GlobalTransform)>,
    pos_query: Query<&GlobalTransform>,
) {
    for (mut cd, mut weapon, target, transform) in query.iter_mut() {
        if cd.not_ready() {
            continue;
        }

        if let Some(target) = target.0 {
            if let Ok(target_transform) = pos_query.get_component::<GlobalTransform>(target) {
                let delta = target_transform.translation - transform.translation;

                if delta.length_squared() > weapon.range * weapon.range {
                    continue;
                }

                // Only fire when target its within weapon cone
                let projection = delta.normalize().dot(transform.local_y().normalize());
                if projection < (weapon.cone / 2.0).cos() {
                    continue;
                }

                weapon.firing = true;
                cd.reset();
            }
        }
    }
}
