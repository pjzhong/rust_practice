use crate::combat::{Target, Team};
use crate::game::GameTimeDelta;
use bevy::prelude::{Entity, GlobalTransform, Query, Res, Vec3};
use multimap::MultiMap;

#[derive(Default)]
pub struct AggroRadius(pub f32);

#[derive(Default)]
pub struct RetargetBehavior {
    pub interval: f32,
    pub remaining_time: f32,
}

pub const HASH_CELL_SIZE: f32 = 50.0;

struct TargetInformation {
    pub entity: Entity,
    pub position: Vec3,
    pub team: Team,
}

struct Targeting {
    pub team: Team,
    pub position: Vec3,
    pub radius: f32,
    pub score: f32,
    pub current_target: Target,
}

impl Targeting {
    fn consider(&mut self, candidate: &TargetInformation) {
        if self.team == candidate.team {
            return;
        }

        let delta = (candidate.position - self.position).length_squared();
        if delta > self.radius.powi(2) {
            return;
        }

        let score = delta;
        if score > self.score {
            return;
        }

        self.score = score;
        self.current_target.0 = Some(candidate.entity);
    }
}

fn get_cell_coordinates(position: Vec3) -> (i32, i32) {
    (
        (position.x / HASH_CELL_SIZE).floor() as i32,
        (position.y / HASH_CELL_SIZE).floor() as i32,
    )
}

pub fn find_targets(
    target_query: Query<(Entity, &GlobalTransform, &Team)>,
    mut targeting_query: Query<(&GlobalTransform, &AggroRadius, &Team, &mut Target)>,
) {
    let mut sorted_targets = MultiMap::new();

    for (entity, transform, team) in target_query.iter() {
        let position = transform.translation;
        sorted_targets.insert(
            get_cell_coordinates(position),
            TargetInformation {
                entity,
                position,
                team: *team,
            },
        );
    }

    //Pick best target for targetter
    for (transform, aggro_radius, team, mut target) in targeting_query.iter_mut() {
        if target.0.is_some() {
            continue;
        }

        let mut targeting = Targeting {
            team: *team,
            position: transform.translation,
            radius: aggro_radius.0,
            score: f32::INFINITY,
            current_target: Target::default(),
        };

        let min_coords = get_cell_coordinates(targeting.position - Vec3::splat(targeting.radius));
        let max_coords = get_cell_coordinates(targeting.position + Vec3::splat(targeting.radius));

        for x in min_coords.0..=max_coords.0 {
            for y in min_coords.0..=max_coords.1 {
                let current_bucket = (x, y);

                match sorted_targets.get_vec(&current_bucket) {
                    None => continue,
                    Some(candidates) => {
                        for candidate in candidates {
                            targeting.consider(candidate);
                        }
                    }
                }
            }
        }

        target.0 = targeting.current_target.0;
    }
}

pub fn do_retargeting(
    dt: Res<GameTimeDelta>,
    mut query: Query<(&mut Target, &mut RetargetBehavior)>,
) {
    for (mut target, mut retarget) in query.iter_mut() {
        retarget.remaining_time -= dt.0;
        if retarget.remaining_time < 0.0 {
            retarget.remaining_time = retarget.interval;
            target.0 = None;
        }
    }
}
