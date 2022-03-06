use bevy::prelude::Vec3;

/// Gets the heading given a delta vector separating two points.
///
/// # Arguments
///
/// * `delta`: difference vector equal to `target - source`.
pub fn get_heading_to_point(delta: Vec3) -> f32 {
    delta.y.atan2(delta.x)
}

/// Returns the smallest angle difference between two stated angles.
///
/// # Arguments
///
/// * `target`: target heading.
///
/// * `initial`: initial heading.
pub fn get_angle_difference(target: f32, initial: f32) -> f32 {
    let a = (target - initial) + std::f32::consts::PI;
    let n = 2.0 * std::f32::consts::PI;
    (a - (a / n).floor() * n) - std::f32::consts::PI
}
