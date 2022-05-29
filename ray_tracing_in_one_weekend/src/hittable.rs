use crate::material::Material;
use crate::ray::Ray;
use crate::Vec3;
use std::rc::Rc;

#[derive(Default)]
pub struct HitRecord {
    pub t: f32,
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
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool;
}
