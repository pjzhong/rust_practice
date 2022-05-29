use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::{Color, Vec3};
use rand::{thread_rng, Rng};

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
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Color, f: f32) -> Self {
        Self {
            albedo,
            fuzz: if f < 1.0 { f } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r: &Ray, rec: &mut HitRecord) -> Option<(Color, Ray)> {
        let reflected = r.dir().normalize().reflect(&rec.normal);
        let scattered = Ray::new(
            rec.p,
            reflected + self.fuzz * Vec3::<f32>::random_in_unit_sphere(),
        );
        let attenuation = self.albedo;
        if scattered.dir().dot_product(&rec.normal) > 0.0 {
            Some((attenuation, scattered))
        } else {
            None
        }
    }
}

pub struct Dielectric {
    ir: f32,
}

impl Dielectric {
    pub fn new(ir: f32) -> Self {
        Self { ir }
    }

    fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 *= r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r: &Ray, rec: &mut HitRecord) -> Option<(Color, Ray)> {
        let refraction_ration = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = r.dir().normalize();
        let cos_theta = (-unit_direction).dot_product(&rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ration * sin_theta > 1.0;
        let direction = if cannot_refract
            || Dielectric::reflectance(cos_theta, refraction_ration) > thread_rng().gen()
        {
            unit_direction.reflect(&rec.normal)
        } else {
            unit_direction.refract(&rec.normal, refraction_ration)
        };

        let scattered = Ray::new(rec.p, direction);
        Some((Color::f32(1.0, 1.0, 1.0), scattered))
    }
}
