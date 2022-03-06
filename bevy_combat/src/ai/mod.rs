use crate::constants::FIXED_TIME_STEP;
use bevy::{core::FixedTimestep, prelude::*};

pub mod aggression;
pub mod idle;
pub mod movement;

#[derive(Default)]
pub struct AIPlugin;

#[derive(PartialEq, Clone, Hash, Debug, Eq, SystemLabel)]
pub enum AISystems {
    PeelManoeuvre,
    Pursue,
    TurnToDestination,
    DoRoaming,
    UpdateAggressionSource,
    Redirecting,
    FindTargets,
    IdleToCombat,
}

impl Plugin for AIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            CoreStage::Update,
            movement::peel_manoeuvre
                .system()
                .with_run_criteria(FixedTimestep::step(FIXED_TIME_STEP as f64))
                .label(AISystems::PeelManoeuvre),
        )
        .add_system_to_stage(
            CoreStage::Update,
            movement::pursue
                .system()
                .with_run_criteria(FixedTimestep::step(FIXED_TIME_STEP as f64))
                .label(AISystems::Pursue),
        )
        .add_system_to_stage(
            CoreStage::Update,
            movement::turn_to_destination
                .system()
                .with_run_criteria(FixedTimestep::step(FIXED_TIME_STEP as f64))
                .label(AISystems::DoRoaming)
                .after(crate::movement::MovementSystems::UpdateHeading)
                .before(crate::movement::MovementSystems::UpdateRotation),
        )
        .add_system_to_stage(
            CoreStage::Update,
            idle::do_roaming
                .system()
                .with_run_criteria(FixedTimestep::step(FIXED_TIME_STEP as f64))
                .label(AISystems::DoRoaming),
        );

        app.add_system_to_stage(
            CoreStage::Update,
            aggression::do_retargeting
                .system()
                .with_run_criteria(FixedTimestep::step(FIXED_TIME_STEP as f64))
                .label(AISystems::Redirecting),
        )
        .add_system_to_stage(
            CoreStage::Update,
            aggression::find_targets
                .system()
                .with_run_criteria(FixedTimestep::step(FIXED_TIME_STEP as f64))
                .label(AISystems::FindTargets),
        );

        app.add_system_to_stage(
            CoreStage::Update,
            idle::idle_to_combat
                .system()
                .with_run_criteria(FixedTimestep::step(FIXED_TIME_STEP as f64))
                .label(AISystems::IdleToCombat),
        );
    }
}
