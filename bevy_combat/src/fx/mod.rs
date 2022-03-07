use crate::fx::animated::{AnimatedEffects, AnimatedEffectsPlugin};
use crate::fx::beams::BeamEffectPlugin;
use bevy::app::{AppBuilder, Plugin};

pub mod animated;
pub mod beams;
pub mod death;

/// An effect that spawns when an effect hits a target.
pub struct HitEffect {
    pub effect: AnimatedEffects,
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(AnimatedEffectsPlugin)
            .add_plugin(BeamEffectPlugin);
    }
}
