use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::{Point, Ray, Vec3, AABB};
use std::sync::Arc;

pub struct XyRectangle {
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    k: f32,
    material: Arc<dyn Material>,
}

impl XyRectangle {
    pub fn new(x0: f32, x1: f32, y0: f32, y1: f32, k: f32, material: Arc<dyn Material>) -> Self {
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

pub struct XzRectangle {
    x0: f32,
    x1: f32,
    z0: f32,
    z1: f32,
    k: f32,
    material: Arc<dyn Material>,
}

impl XzRectangle {
    pub fn new(x0: f32, x1: f32, z0: f32, z1: f32, k: f32, material: Arc<dyn Material>) -> Self {
        Self {
            x0,
            x1,
            z0,
            z1,
            k,
            material,
        }
    }
}

impl Hittable for XzRectangle {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - r.origin().y) / r.dir().y;
        if t < t_min || t_max < t {
            return None;
        }

        let x = r.origin().x + t * r.dir().x;
        let z = r.origin().z + t * r.dir().z;
        if x < self.x0 || self.x1 < x || z < self.z0 || self.z1 < z {
            return None;
        }

        let (front_face, normal) = HitRecord::calc_face_normal(r, &Vec3::f32(0.0, 1.0, 0.0));
        Some(HitRecord {
            u: (x - self.x0) / (self.x1 - self.x0),
            v: (z - self.z0) / (self.z1 - self.z0),
            t,
            front_face,
            normal,
            p: r.at(t),
            material: self.material.clone(),
        })
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(
            Vec3::f32(self.x0, self.k - 0.0001, self.z0),
            Vec3::f32(self.x1, self.k + 0.0001, self.z1),
        ))
    }
}

pub struct YzRectangle {
    y0: f32,
    y1: f32,
    z0: f32,
    z1: f32,
    k: f32,
    material: Arc<dyn Material>,
}

impl YzRectangle {
    pub fn new(y0: f32, y1: f32, z0: f32, z1: f32, k: f32, material: Arc<dyn Material>) -> Self {
        Self {
            y0,
            y1,
            z0,
            z1,
            k,
            material,
        }
    }
}

impl Hittable for YzRectangle {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - r.origin().x) / r.dir().x;
        if t < t_min || t_max < t {
            return None;
        }

        let y = r.origin().y + t * r.dir().y;
        let z = r.origin().z + t * r.dir().z;
        if y < self.y0 || self.y1 < y || z < self.z0 || self.z1 < z {
            return None;
        }

        let (front_face, normal) = HitRecord::calc_face_normal(r, &Vec3::f32(1.0, 0.0, 0.0));
        Some(HitRecord {
            u: (y - self.y0) / (self.y1 - self.y0),
            v: (z - self.z0) / (self.z1 - self.z0),
            t,
            front_face,
            normal,
            p: r.at(t),
            material: self.material.clone(),
        })
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(
            Point::f32(self.k - 0.0001, self.y0, self.z0),
            Point::f32(self.k + 0.0001, self.y1, self.z1),
        ))
    }
}
