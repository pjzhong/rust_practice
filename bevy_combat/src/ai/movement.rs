use crate::ai::idle::IdleBehavior;
use crate::combat::Target;
use crate::constants::FIXED_TIME_STEP;
use crate::math_util::{get_angle_difference, get_heading_to_point};
use crate::movement::{Heading, MaxTurnSpeed, TurnSpeed};
use bevy::prelude::{Commands, Entity, GlobalTransform, Query, Vec3, With};

#[derive(Default)]
pub struct PursueBehavior;

pub const PROXIMITY_RADIUS: f32 = 64.0;

#[derive(Default)]
pub struct TurnToDestinationBehavior {
    pub destination: Vec3,
}

pub fn pursue(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Target,
            &GlobalTransform,
            &mut TurnToDestinationBehavior,
        ),
        With<PursueBehavior>,
    >,
    pos_query: Query<&GlobalTransform>,
) {
    for (entity, target, transform, mut turn_to) in query.iter_mut() {
        if let Some(target) = target.0 {
            match pos_query.get_component::<GlobalTransform>(target) {
                Err(_) => {
                    // target does not have position. Go to idel state
                    commands.entity(entity).remove::<PursueBehavior>();
                    commands.entity(entity).insert(IdleBehavior);
                }
                Ok(target_transform) => {
                    turn_to.destination = target_transform.translation;

                    let delta =
                        (target_transform.translation - transform.translation).length_squared();
                    if delta < PROXIMITY_RADIUS * PROXIMITY_RADIUS {
                        commands
                            .entity(entity)
                            .remove_bundle::<(TurnToDestinationBehavior, PursueBehavior)>();
                        commands.entity(entity).insert(PeelManoeuvreBehavior);
                    }
                }
            }
        }
    }
}

pub struct PeelManoeuvreBehavior;

const ENGAGEMENT_RADIUS: f32 = 64.0;

pub fn peel_manoeuvre(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &Target,
            &GlobalTransform,
            &Heading,
            &MaxTurnSpeed,
            &mut TurnSpeed,
        ),
        With<PeelManoeuvreBehavior>,
    >,
    pos_query: Query<&GlobalTransform>,
) {
    for (entity, target, transform, heading, max_turn_speed, mut turn_speed) in query.iter_mut() {
        if let Some(target) = target.0 {
            match pos_query.get_component::<GlobalTransform>(target) {
                Err(_) => {}
                Ok(target_transform) => {
                    let mut delta = target_transform.translation - transform.translation;
                    delta.z = 0.0;
                    let angle_diff =
                        get_angle_difference(get_heading_to_point(delta), heading.radians);

                    if angle_diff.abs() < 0.3 * std::f32::consts::PI {
                        turn_speed.radians_per_second =
                            -max_turn_speed.radians_per_second * angle_diff.signum();
                    } else {
                        turn_speed.radians_per_second = 0.0;
                    }

                    if delta.length_squared() > ENGAGEMENT_RADIUS * ENGAGEMENT_RADIUS {
                        commands.entity(entity).remove::<PeelManoeuvreBehavior>();
                        commands
                            .entity(entity)
                            .insert(PursueBehavior)
                            .insert(TurnToDestinationBehavior::default());
                    }
                }
            }
        }
    }
}

pub fn turn_to_destination(
    mut query: Query<(
        &TurnToDestinationBehavior,
        &GlobalTransform,
        &MaxTurnSpeed,
        &Heading,
        &mut TurnSpeed,
    )>,
) {
    for (behavior, transform, max_turn_speed, heading, mut turn_speed) in query.iter_mut() {
        let delta = behavior.destination - transform.translation;
        let desired_heading = get_heading_to_point(delta);

        //计算他们的角度差，这个是什么数学公式
        //如果两个对象都是面对面的，则每秒旋转的角度接近0
        let diff = get_angle_difference(desired_heading, heading.radians);
        turn_speed.radians_per_second = diff.signum()
            * max_turn_speed
                .radians_per_second
                .min(diff.abs() / FIXED_TIME_STEP);
    }
}
