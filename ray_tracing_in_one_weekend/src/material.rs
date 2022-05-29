use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::{Color, Vec3};

pub trait Material {
    fn scatter(&self, r: &Ray, rec: &mut HitRecord) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _: &Ray, rec: &mut HitRecord) -> Option<(Color, Ray)> {
        let scatter_direction = {
            let mut t = rec.normal + Vec3::<f32>::random_unit_vecotr();
            if t.near_zero() {
                t = rec.normal;
            }
            t
        };

        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}

pub struct Metal {
    albedo: Color,
}

impl Metal {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Metal {
    fn scatter(&self, r: &Ray, rec: &mut HitRecord) -> Option<(Color, Ray)> {
        let reflected = r.dir().normalize().reflect(&rec.normal);
        let scattered = Ray::new(rec.p, reflected);
        let attenuation = self.albedo;
        if scattered.dir().dot_product(&rec.normal) > 0.0 {
            Some((attenuation, scattered))
        } else {
            None
        }
    }
}
