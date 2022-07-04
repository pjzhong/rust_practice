extern crate core;

use hittable::FlipNormals;

use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::rectangle::{AARect, Plane};
use crate::vec::{Axis, Vec3};
use std::mem::swap;

pub mod bvh;
pub mod camera;
pub mod hittable;
pub mod material;
pub mod perlin;
pub mod ray;
pub mod rectangle;
pub mod sphere;
pub mod texture;
pub mod vec;

pub type Color = Vec3<f32>;
pub type Point = Vec3<f32>;
pub type Image = Vec<Vec<Vec3<f32>>>;

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min {
        min
    } else if max < x {
        max
    } else {
        x
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    minimum: Vec3<f32>,
    maximum: Vec3<f32>,
}

impl AABB {
    pub fn new(minimum: Vec3<f32>, maximum: Vec3<f32>) -> Self {
        Self { minimum, maximum }
    }

    pub fn min(&self) -> &Vec3<f32> {
        &self.minimum
    }

    pub fn max(&self) -> &Vec3<f32> {
        &self.maximum
    }

    pub fn hit(&self, r: &Ray, mut t_min: f32, mut t_max: f32) -> bool {
        for a in Axis::values() {
            let inv_d = 1.0 / r.dir()[a];
            let mut t0 = (self.minimum[a] - r.origin()[a]) * inv_d;
            let mut t1 = (self.maximum[a] - r.origin()[a]) * inv_d;
            if inv_d < 0.0 {
                swap(&mut t0, &mut t1);
            }

            t_min = t_min.max(t0);
            t_max = t_max.min(t1);
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn surrounding_box(box0: &AABB, box1: &AABB) -> Self {
        let minimum = Vec3::f32(
            box0.minimum.x.min(box1.minimum.x),
            box0.minimum.y.min(box1.minimum.y),
            box0.minimum.z.min(box1.minimum.z),
        );
        let maximum = Vec3::f32(
            box0.maximum.x.max(box1.maximum.x),
            box0.maximum.y.max(box1.maximum.y),
            box0.maximum.z.max(box1.maximum.z),
        );

        Self { minimum, maximum }
    }
}

pub struct Cube {
    box_min: Point,
    box_max: Point,
    sides: Vec<Box<dyn Hittable>>,
}

impl Cube {
    pub fn new<M: Material + Clone + 'static>(min: &Point, max: &Point, material: M) -> Self {
        Self {
            sides: vec![
                Box::new(AARect::new(
                    Plane::XY,
                    min.x,
                    max.x,
                    min.y,
                    max.y,
                    max.z,
                    material.clone(),
                )),
                Box::new(FlipNormals(AARect::new(
                    Plane::XY,
                    min.x,
                    max.x,
                    min.y,
                    max.y,
                    min.z,
                    material.clone(),
                ))),
                Box::new(AARect::new(
                    Plane::XZ,
                    min.x,
                    max.x,
                    min.z,
                    max.z,
                    max.y,
                    material.clone(),
                )),
                Box::new(FlipNormals(AARect::new(
                    Plane::XZ,
                    min.x,
                    max.x,
                    min.z,
                    max.z,
                    min.y,
                    material.clone(),
                ))),
                Box::new(AARect::new(
                    Plane::YZ,
                    min.y,
                    max.y,
                    min.z,
                    max.z,
                    max.x,
                    material.clone(),
                )),
                Box::new(FlipNormals(AARect::new(
                    Plane::YZ,
                    min.y,
                    max.y,
                    min.z,
                    max.z,
                    min.x,
                    material,
                ))),
            ],
            box_min: *min,
            box_max: *max,
        }
    }
}

impl Hittable for Cube {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.sides.hit(r, t_min, t_max)
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(self.box_min, self.box_max))
    }
}
