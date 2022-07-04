use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::vec::Axis;
use crate::{Ray, Vec3, AABB};

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
        Self {
            a0,
            a1,
            b0,
            b1,
            k,
            plane,
            material,
        }
    }
}

impl<M: Material> Hittable for AARect<M> {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let (k_axis, a_axis, b_axis) = match self.plane {
            Plane::YZ => (Axis::X, Axis::Y, Axis::Z),
            Plane::XZ => (Axis::Y, Axis::X, Axis::Z),
            Plane::XY => (Axis::Z, Axis::X, Axis::Y),
        };
        let t = (self.k - r.origin()[k_axis]) / r.dir()[k_axis];
        if t < t_min || t_max < t {
            None
        } else {
            let a = r.origin()[a_axis] + t * r.dir()[a_axis];
            let b = r.origin()[b_axis] + t * r.dir()[b_axis];
            if a < self.a0 || self.a1 < a || b < self.b0 || self.b1 < b {
                None
            } else {
                Some(HitRecord {
                    u: (a - self.a0) / (self.a1 - self.a0),
                    v: (b - self.b0) / (self.b1 - self.b0),
                    t,
                    normal: {
                        let mut v = Vec3::f32(0.0, 0.0, 0.0);
                        v[k_axis] = 1.0;
                        v
                    },
                    p: r.at(t),
                    material: &self.material,
                })
            }
        }
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        let (k_axis, a_axis, b_axis) = match self.plane {
            Plane::YZ => (Axis::X, Axis::Y, Axis::Z),
            Plane::XZ => (Axis::Y, Axis::X, Axis::Z),
            Plane::XY => (Axis::Z, Axis::X, Axis::Y),
        };

        let mut min = Vec3::<f32>::default();
        let mut max = Vec3::<f32>::default();

        min[k_axis] = self.k - 0.0001;
        min[a_axis] = self.a0;
        min[b_axis] = self.b0;

        max[k_axis] = self.k + 0.0001;
        max[a_axis] = self.a1;
        max[b_axis] = self.b1;

        Some(AABB::new(min, max))
    }
}
