use crate::vec::Vec3;

pub mod camera;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod sphere;
pub mod vec;

pub type Color = Vec3<f32>;

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min {
        min
    } else if max < x {
        max
    } else {
        x
    }
}
