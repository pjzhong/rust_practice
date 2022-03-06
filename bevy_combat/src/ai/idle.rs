use crate::ai::movement::{PursueBehavior, TurnToDestinationBehavior};
use crate::combat::Target;
use bevy::prelude::{Changed, Commands, Entity, GlobalTransform, Query, Vec3, With, Without};
use rand::Rng;

pub struct IdleBehavior;

pub struct RoamBehavior {
    pub centre: Vec3,
    pub radius: f32,
}

pub const ARRIVEAL_TOLERANCE: f32 = 10.0;

pub fn do_roaming(
    mut query: Query<(
        &GlobalTransform,
        &RoamBehavior,
        &mut TurnToDestinationBehavior),
        With<IdleBehavior>,
    >,
) {
    let mut rng = rand::thread_rng();

    for (transform, roam, mut turn_to_destination) in query.iter_mut() {
        let delta = (turn_to_destination.destination - transform.translation).length_squared();

        if delta > ARRIVEAL_TOLERANCE * ARRIVEAL_TOLERANCE {
            let radius = rng.gen_range(0.0..roam.radius);
            let angle = 2.0 * rng.gen_range(0.0..std::f32::consts::PI);
            let new_target =
                roam.centre + radius * angle.cos() * Vec3::X + radius * angle.sin() * Vec3::Y;
            turn_to_destination.destination = new_target;
        }
    }
}

pub fn idle_to_combat(
    mut commands: Commands,
    query: Query<(Entity, &Target), (With<IdleBehavior>, Without<PursueBehavior>, Changed<Target>)>,
) {
    for (entity, target) in query.iter() {
        if target.0.is_some() {
            commands.entity(entity).insert(PursueBehavior::default());
            commands.entity(entity).remove::<IdleBehavior>();
        }
    }
}
