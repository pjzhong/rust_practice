use crate::material::Material;
use crate::ray::Ray;
use crate::{Vec3, AABB};
use std::rc::Rc;

#[derive(Default)]
pub struct HitRecord {
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub p: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub material: Option<Rc<dyn Material>>,
    pub front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3<f32>) {
        self.front_face = r.dir().dot_product(outward_normal) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB>;
}

impl Hittable for [Rc<dyn Hittable>] {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closet_so_far = t_max;
        let mut record = None;

        for hittable in self {
            if let Some(rec) = hittable.hit(r, t_min, closet_so_far) {
                closet_so_far = rec.t;
                record.replace(rec);
            }
        }

        record
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        if self.is_empty() {
            return None;
        }

        let mut result = None;
        for hittable in self {
            if let Some(aabb) = hittable.bounding_box(time0, time1) {
                if let Some(prev) = result {
                    result = Some(AABB::surrounding_box(&prev, &aabb));
                } else {
                    result = Some(aabb);
                }
            }
        }

        result
    }
}
