use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::Vec3;
use std::rc::Rc;

pub struct Sphere {
    radius: f32,
    time1: f32,
    time2: f32,
    center1: Vec3<f32>,
    center2: Vec3<f32>,
    material: Rc<dyn Material>,
}

impl Sphere {
    pub fn new(
        center1: Vec3<f32>,
        center2: Vec3<f32>,
        radius: f32,
        time1: f32,
        time2: f32,
        material: Rc<dyn Material>,
    ) -> Self {
        Self {
            center1,
            center2,
            radius,
            material,
            time1,
            time2,
        }
    }

    pub fn center(&self, time: f32) -> Vec3<f32> {
        self.center1
            + ((time - self.time1) / (self.time2 - self.time1)) * (self.center2 - self.center1)
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        let center = self.center(r.time());
        let oc = r.origin() - &center;
        let a = r.dir().length_squared();
        let half_b = oc.dot_product(r.dir());
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return false;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return false;
            }
        }

        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal = (rec.p - center) / self.radius;
        rec.set_face_normal(r, &outward_normal);
        rec.material = Some(self.material.clone());
        true
    }
}
