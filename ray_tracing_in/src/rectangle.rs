use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::{Ray, Vec3, AABB};
use std::rc::Rc;

pub struct XyRectangle {
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    k: f32,
    material: Rc<dyn Material>,
}

impl XyRectangle {
    pub fn new(x0: f32, x1: f32, y0: f32, y1: f32, k: f32, material: Rc<dyn Material>) -> Self {
        Self {
            x0,
            x1,
            y0,
            y1,
            k,
            material,
        }
    }
}

impl Hittable for XyRectangle {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - r.origin().z) / r.dir().z;
        if t < t_min || t_max < t {
            return None;
        }

        let x = r.origin().x + t * r.dir().x;
        let y = r.origin().y + t * r.dir().y;
        if x < self.x0 || self.x1 < x || y < self.y0 || self.y1 < y {
            return None;
        }

        let (front_face, normal) = HitRecord::calc_face_normal(r, &Vec3::f32(0.0, 0.0, 1.0));
        Some(HitRecord {
            u: (x - self.x0) / (self.x1 - self.x0),
            v: (y - self.y0) / (self.y1 - self.y0),
            t,
            front_face,
            normal,
            p: r.at(t),
            material: self.material.clone(),
        })
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(
            Vec3::f32(self.x0, self.y0, self.k - 0.0001),
            Vec3::f32(self.x1, self.y1, self.k + 0.0001),
        ))
    }
}
