use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::texture::Texture;
use crate::{Color, Point, Vec3};
use rand::{thread_rng, Rng};

pub trait Material: Sync + Send {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;

    fn emitted(&self, _: f32, _: f32, _: &Point) -> Color {
        Color::f32(0.0, 0.0, 0.0)
    }
}

#[derive(Clone)]
pub struct Lambertian<T: Texture> {
    albedo: T,
}

impl<T: Texture> Lambertian<T> {
    pub fn new(albedo: T) -> Self {
        Self { albedo }
    }
}

impl<T: Texture> Material for Lambertian<T> {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Color, Ray)> {
        let target = hit.p + hit.normal + Vec3::<f32>::random_in_unit_sphere();
        let scattered = Ray::new(hit.p, target - hit.p, ray.time());
        Some((self.albedo.value(hit.u, hit.v, &hit.p), scattered))
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = r_in.dir().normalize().reflect(&rec.normal);
        let scattered = Ray::new(
            rec.p,
            reflected + self.fuzz * Vec3::<f32>::random_in_unit_sphere(),
            r_in.time(),
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
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Color, Ray)> {
        let (outward_normal, ni_over_nt, cosine) = if ray.dir().dot_product(&hit.normal) > 0. {
            (
                -hit.normal,
                self.ir,
                self.ir * ray.dir().dot_product(&hit.normal) / ray.dir().length(),
            )
        } else {
            (
                hit.normal,
                1.0 / self.ir,
                -ray.dir().dot_product(&hit.normal) / ray.dir().length(),
            )
        };

        let direction = ray
            .dir()
            .refract(&outward_normal, ni_over_nt)
            .filter(|_| thread_rng().gen::<f32>() >= schlick(cosine, self.ir))
            .unwrap_or_else(|| ray.dir().reflect(&hit.normal));

        let attenuation = Vec3::f32(1., 1., 1.);
        let ray = Ray::new(hit.p, direction, ray.time());
        Some((attenuation, ray))
    }
}

pub struct DiffuseLight<T: Texture> {
    emit: T,
}

impl<T: Texture> DiffuseLight<T> {
    pub fn new(emit: T) -> Self {
        Self { emit }
    }
}

impl<T: Texture> Material for DiffuseLight<T> {
    fn scatter(&self, _: &Ray, _: &HitRecord) -> Option<(Color, Ray)> {
        None
    }

    fn emitted(&self, u: f32, v: f32, p: &Point) -> Color {
        self.emit.value(u, v, p)
    }
}

/// [Schlick's approximation][schlick] for computing reflection vs. refraction
/// at a material surface.
///
/// [schlick]: https://en.wikipedia.org/wiki/Schlick%27s_approximation
#[inline]
fn schlick(cos: f32, ref_idx: f32) -> f32 {
    let r0 = (1. - ref_idx) / (1. + ref_idx);
    let r0 = r0 * r0;
    r0 + (1. - r0) * f32::powf(1. - cos, 5.)
}
