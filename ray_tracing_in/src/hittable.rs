use crate::material::Material;
use crate::ray::Ray;
use crate::texture::Texture;
use crate::{Axis, Color, Vec3, AABB};
use rand::Rng;

pub struct HitRecord<'a> {
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub p: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub material: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    pub fn calc_face_normal(r: &Ray, outward_normal: &Vec3<f32>) -> (bool, Vec3<f32>) {
        let front_face = r.dir().dot_product(outward_normal) < 0.0;
        let normal = if front_face {
            *outward_normal
        } else {
            -outward_normal
        };

        (front_face, normal)
    }
}

pub trait Hittable: Sync + Send {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB>;
}

impl Hittable for [Box<dyn Hittable>] {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closet_so_far = t_max;
        let mut record = None;

        for hittable in self {
            if let Some(hit) = hittable.hit(r, t_min, closet_so_far) {
                closet_so_far = hit.t;
                record = Some(hit);
            }
        }

        record
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        if self.is_empty() {
            return None;
        }

        let mut result = None;
        for hittable in self {
            if let Some(aabb) = hittable.bounding_box(time0, time1) {
                if let Some(prev) = result {
                    result = Some(AABB::surrounding_box(&prev, &aabb));
                } else {
                    result = Some(aabb);
                }
            }
        }

        result
    }
}

pub struct Translate<H: Hittable> {
    offset: Vec3<f32>,
    hittable: H,
}

impl<H: Hittable> Translate<H> {
    pub fn new(hittable: H, offset: Vec3<f32>) -> Self {
        Self { offset, hittable }
    }
}

impl<H: Hittable> Hittable for Translate<H> {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let move_r = Ray::new(*r.origin() - self.offset, *r.dir(), r.time());
        self.hittable.hit(&move_r, t_min, t_max).map(|mut hit| {
            hit.p += self.offset;
            hit
        })
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        self.hittable
            .bounding_box(time0, time1)
            .map(|aabb| AABB::new(aabb.minimum + self.offset, aabb.maximum + self.offset))
    }
}

pub struct RotateY<H: Hittable> {
    sin_theta: f32,
    cos_theta: f32,
    aabb: Option<AABB>,
    hittable: H,
}

impl<H: Hittable> RotateY<H> {
    pub fn new(hittable: H, angle: f32) -> Self {
        let radians = angle.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let aabb = hittable.bounding_box(0.0, 1.0);

        let mut min = Vec3::f32(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::f32(f32::MIN, f32::MIN, f32::MIN);
        if let Some(aabb) = &aabb {
            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        let (i, j, k) = (i as f32, j as f32, k as f32);
                        let x = i * aabb.maximum.x + (1.0 - i) * aabb.minimum.x;
                        let y = j * aabb.maximum.y + (1.0 - j) * aabb.minimum.y;
                        let z = k * aabb.maximum.z + (1.0 - k) * aabb.minimum.z;

                        let newx = cos_theta * x + sin_theta * z;
                        let newz = -sin_theta * x + cos_theta * z;

                        let tester = Vec3::f32(newx, y, newz);

                        for c in Axis::values() {
                            min[c] = min[c].min(tester[c]);
                            max[c] = max[c].max(tester[c]);
                        }
                    }
                }
            }
        }

        Self {
            sin_theta,
            cos_theta,
            aabb: Some(AABB::new(min, max)),
            hittable,
        }
    }
}

impl<H: Hittable> Hittable for RotateY<H> {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let origin = Vec3::f32(
            self.cos_theta * r.origin().x - self.sin_theta * r.origin().z,
            r.origin().y,
            self.sin_theta * r.origin().x + self.cos_theta * r.origin().z,
        );
        let direction = Vec3::f32(
            self.cos_theta * r.dir().x - self.sin_theta * r.dir().z,
            r.dir().y,
            self.sin_theta * r.dir().x + self.cos_theta * r.dir().z,
        );

        let rotated_r = Ray::new(origin, direction, r.time());
        if let Some(hit) = self.hittable.hit(&rotated_r, t_min, t_max) {
            let p = Vec3::f32(
                self.cos_theta * hit.p.x + self.sin_theta * hit.p.z,
                hit.p.y,
                -self.sin_theta * hit.p.x + self.cos_theta * hit.p.z,
            );
            let normal = Vec3::f32(
                self.cos_theta * hit.normal.x + self.sin_theta * hit.normal.z,
                hit.normal.y,
                -self.sin_theta * hit.normal.x + self.cos_theta * hit.normal.z,
            );
            Some(HitRecord {
                t: hit.t,
                u: hit.u,
                v: hit.v,
                p,
                normal,
                material: hit.material,
            })
        } else {
            None
        }
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        self.aabb
    }
}

pub struct ConstantMedium<H: Hittable, M: Material> {
    neg_inv_density: f32,
    boundary: H,
    material: M,
}

impl<H: Hittable, M: Material> ConstantMedium<H, M> {
    pub fn new(boundary: H, neg_inv_density: f32, material: M) -> Self {
        Self {
            boundary,
            neg_inv_density: -(1.0 / neg_inv_density),
            material,
        }
    }
}

impl<H: Hittable, M: Material> Hittable for ConstantMedium<H, M> {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if let Some(hit1) = self.boundary.hit(r, f32::MIN, f32::MAX) {
            if let Some(hit2) = self.boundary.hit(r, hit1.t + 0.001, f32::MAX) {
                let (hit1t, hit2t) = (hit1.t.max(t_min), hit2.t.min(t_max));
                if hit2t <= hit1t {
                    return None;
                }

                let mut rang = rand::thread_rng();
                let ray_length = r.dir().length();
                let distance_inside_boundary = (hit2t - hit1t) * ray_length;
                let hit_distance = self.neg_inv_density * rang.gen::<f32>().ln();

                if hit_distance < distance_inside_boundary {
                    let t = hit1t + hit_distance / ray_length;
                    return Some(HitRecord {
                        t,
                        u: hit1.u,
                        v: hit1.v,
                        p: r.at(t),
                        material: &self.material,
                        normal: Vec3::f32(1.0, 0.0, 0.0),
                    });
                }
            }
        }
        None
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        self.boundary.bounding_box(time0, time1)
    }
}
pub struct Isotropic<T: Texture> {
    pub albedo: T,
}

impl<T: Texture> Isotropic<T> {
    pub fn new(albedo: T) -> Self {
        Self { albedo }
    }
}

impl<T: Texture> Material for Isotropic<T> {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        Some((
            self.albedo.value(rec.u, rec.v, &rec.p),
            Ray::new(rec.p, Vec3::<f32>::random_in_unit_sphere(), r.time()),
        ))
    }
}

pub struct FlipNormals<H>(pub H);

impl<H: Hittable> Hittable for FlipNormals<H> {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.0.hit(r, t_min, t_max).map(|hit| HitRecord {
            normal: -hit.normal,
            ..hit
        })
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        self.0.bounding_box(time0, time1)
    }
}
