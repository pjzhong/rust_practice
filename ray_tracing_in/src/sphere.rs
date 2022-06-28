use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::{Point, Vec3, AABB};
use std::f32::consts::PI;
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
    pub fn steady(center1: Vec3<f32>, radius: f32, material: Rc<dyn Material>) -> Self {
        Self {
            center1,
            center2: center1,
            radius,
            material,
            time1: 0.0,
            time2: 1.0,
        }
    }

    pub fn motion(
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
        if self.center1 == self.center2 {
            self.center1
        } else {
            self.center1
                + ((time - self.time1) / (self.time2 - self.time1)) * (self.center2 - self.center1)
        }
    }

    fn calc_sphere_uv(p: &Point) -> (f32, f32) {
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;

        (phi / (2.0 * PI), theta / PI)
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let center = self.center(r.time());
        let oc = r.origin() - &center;
        let a = r.dir().length_squared();
        let half_b = oc.dot_product(r.dir());
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let p = r.at(root);
        let outward_normal = (p - center) / self.radius;
        let (u, v) = Sphere::calc_sphere_uv(&outward_normal);
        let (front_face, normal) = HitRecord::calc_face_normal(r, &outward_normal);
        Some(HitRecord {
            u,
            v,
            t: root,
            front_face,
            normal,
            p,
            material: self.material.clone(),
        })
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        let offset = Vec3::f32(self.radius, self.radius, self.radius);
        if self.center1 == self.center2 {
            Some(AABB::new(self.center1 - offset, self.center1 + offset))
        } else {
            let box0 = AABB::new(self.center(time0) - offset, self.center(time0) + offset);
            let box1 = AABB::new(self.center(time1) - offset, self.center(time1) + offset);

            Some(AABB::surrounding_box(&box0, &box1))
        }
    }
}
