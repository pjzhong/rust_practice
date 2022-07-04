use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec::{Axis, Vec3};
use crate::AABB;

pub enum Plane {
    YZ,
    XZ,
    XY,
}

pub struct AARect<M: Material> {
    plane: Plane,
    a0: f32,
    a1: f32,
    b0: f32,
    b1: f32,
    k: f32,
    material: M,
}

impl<M: Material> AARect<M> {
    pub fn new(plane: Plane, a0: f32, a1: f32, b0: f32, b1: f32, k: f32, material: M) -> Self {
        AARect {
            plane,
            a0,
            a1,
            b0,
            b1,
            k,
            material,
        }
    }
}

impl<M: Material> Hittable for AARect<M> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let (k_axis, a_axis, b_axis) = match &self.plane {
            Plane::YZ => (Axis::X, Axis::Y, Axis::Z),
            Plane::XZ => (Axis::Y, Axis::X, Axis::Z),
            Plane::XY => (Axis::Z, Axis::X, Axis::Y),
        };
        let t = (self.k - ray.origin()[k_axis]) / ray.dir()[k_axis];
        if t < t_min || t > t_max {
            None
        } else {
            let a = ray.origin()[a_axis] + t * ray.dir()[a_axis];
            let b = ray.origin()[b_axis] + t * ray.dir()[b_axis];
            if a < self.a0 || a > self.a1 || b < self.b0 || b > self.b1 {
                None
            } else {
                let u = (a - self.a0) / (self.a1 - self.a0);
                let v = (b - self.b0) / (self.b1 - self.b0);
                let p = ray.at(t);
                let mut normal = Vec3::<f32>::default();
                normal[k_axis] = 1.0;
                Some(HitRecord {
                    t,
                    u,
                    v,
                    p,
                    normal,
                    material: &self.material,
                })
            }
        }
    }

    fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        let (k_axis, a_axis, b_axis) = match &self.plane {
            Plane::YZ => (Axis::X, Axis::Y, Axis::Z),
            Plane::XZ => (Axis::Y, Axis::X, Axis::Z),
            Plane::XY => (Axis::Z, Axis::X, Axis::Y),
        };
        let mut minimum = Vec3::default();
        let mut maximum = Vec3::default();

        minimum[k_axis] = self.k - 0.0001;
        minimum[a_axis] = self.a0;
        minimum[b_axis] = self.b0;

        maximum[k_axis] = self.k + 0.0001;
        maximum[a_axis] = self.a0;
        maximum[b_axis] = self.a1;
        Some(AABB::new(minimum, maximum))
    }
}
