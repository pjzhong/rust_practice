use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::rectangle::{XyRectangle, XzRectangle, YzRectangle};
use crate::vec::Vec3;
use std::mem::swap;
use std::sync::Arc;

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

#[derive(Copy, Clone)]
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

    pub fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / r.dir()[a];
            let mut t0 = (self.min()[a] - r.origin()[a]) * inv_d;
            let mut t1 = (self.max()[a] - r.origin()[a]) * inv_d;
            if inv_d < 0.0 {
                swap(&mut t0, &mut t1);
            }

            let t_min = t_min.max(t0);
            let t_max = t_max.max(t1);
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn surrounding_box(box0: &AABB, box1: &AABB) -> Self {
        let minimum = Vec3::f32(
            box0.min().x.min(box1.min().x),
            box0.min().y.min(box1.min().y),
            box0.min().z.min(box1.min().z),
        );
        let maximum = Vec3::f32(
            box0.max().x.max(box1.max().x),
            box0.max().y.max(box1.max().y),
            box0.max().z.max(box1.max().z),
        );

        Self { minimum, maximum }
    }
}

pub struct Boxes {
    box_min: Point,
    box_max: Point,
    sides: Vec<Box<dyn Hittable>>,
}

impl Boxes {
    pub fn new(p0: &Point, p1: &Point, material: Arc<dyn Material>) -> Self {
        Self {
            sides: vec![
                Box::new(XyRectangle::new(
                    p0.x,
                    p1.x,
                    p0.y,
                    p1.y,
                    p1.z,
                    material.clone(),
                )),
                Box::new(XyRectangle::new(
                    p0.x,
                    p1.x,
                    p0.y,
                    p1.y,
                    p0.z,
                    material.clone(),
                )),
                Box::new(XzRectangle::new(
                    p0.x,
                    p1.x,
                    p0.z,
                    p1.z,
                    p1.y,
                    material.clone(),
                )),
                Box::new(XzRectangle::new(
                    p0.x,
                    p1.x,
                    p0.z,
                    p1.z,
                    p0.y,
                    material.clone(),
                )),
                Box::new(YzRectangle::new(
                    p0.y,
                    p1.y,
                    p0.z,
                    p1.z,
                    p1.x,
                    material.clone(),
                )),
                Box::new(YzRectangle::new(
                    p0.y,
                    p1.y,
                    p0.z,
                    p1.z,
                    p0.x,
                    material.clone(),
                )),
            ],
            box_min: *p0,
            box_max: *p1,
        }
    }
}

impl Hittable for Boxes {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.sides.hit(r, t_min, t_max)
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(self.box_min, self.box_max))
    }
}
