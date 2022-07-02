use crate::material::Material;
use crate::ray::Ray;
use crate::texture::{SolidColor, Texture};
use crate::{Color, Vec3, AABB};
use rand::Rng;
use std::sync::Arc;

pub struct HitRecord {
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub p: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub material: Arc<dyn Material>,
    pub front_face: bool,
}

impl HitRecord {
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
            if let Some(rec) = hittable.hit(r, t_min, closet_so_far) {
                closet_so_far = rec.t;
                record.replace(rec);
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

pub struct Translate {
    offset: Vec3<f32>,
    hittable: Box<dyn Hittable>,
}

impl Translate {
    pub fn new(hittable: Box<dyn Hittable>, offset: Vec3<f32>) -> Self {
        Self { offset, hittable }
    }
}

impl Hittable for Translate {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let move_r = Ray::new(*r.origin() - self.offset, *r.dir(), r.time());
        self.hittable.hit(&move_r, t_min, t_max).map(|mut rec| {
            rec.p += self.offset;
            (rec.front_face, rec.normal) = HitRecord::calc_face_normal(&move_r, &rec.normal);

            rec
        })
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        self.hittable
            .bounding_box(time0, time1)
            .map(|aabb| AABB::new(aabb.minimum + self.offset, aabb.maximum + self.offset))
    }
}

pub struct RotateY {
    sin_theta: f32,
    cos_theta: f32,
    aabb: Option<AABB>,
    hittable: Box<dyn Hittable>,
}

impl RotateY {
    pub fn new(hittable: Box<dyn Hittable>, angle: f32) -> Self {
        let radians = angle.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let aabb = hittable.bounding_box(0.0, 1.0);

        let mut min = Vec3::f32(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = -Vec3::f32(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        if let Some(aabb) = &aabb {
            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        let (i, j, k) = (i as f32, j as f32, k as f32);
                        let x = i * aabb.max().x + (1.0 - i) * aabb.min().x;
                        let y = j * aabb.max().y + (1.0 - j) * aabb.min().y;
                        let z = k * aabb.max().z + (1.0 - k) * aabb.min().z;

                        let newx = cos_theta * x + sin_theta * z;
                        let newz = -sin_theta * x + cos_theta * z;

                        let tester = Vec3::f32(newx, y, newz);

                        for c in 0..3 {
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

impl Hittable for RotateY {
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
            let (front_face, normal) = HitRecord::calc_face_normal(&rotated_r, &normal);
            Some(HitRecord {
                t: hit.t,
                u: hit.u,
                v: hit.v,
                p,
                normal,
                front_face,
                material: hit.material.clone(),
            })
        } else {
            None
        }
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        self.aabb
    }
}

pub struct ConstantMedium {
    neg_inv_density: f32,
    boundary: Box<dyn Hittable>,
    phase_function: Arc<dyn Material>,
}

impl ConstantMedium {
    pub fn new(boundary: Box<dyn Hittable>, neg_inv_density: f32, c: Color) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / neg_inv_density,
            phase_function: Arc::new(Isotropic::new(c)),
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if let Some(rec1) = self.boundary.hit(r, -f32::INFINITY, f32::INFINITY) {
            if let Some(rec2) = self.boundary.hit(r, rec1.t + 0.001, f32::INFINITY) {
                let (mut rec1_t, mut rec2_t) = (rec1.t, rec2.t);
                if rec1_t < t_min {
                    rec1_t = t_min;
                }
                if t_max < rec2_t {
                    rec2_t = t_max
                }

                if rec2_t <= rec1_t {
                    return None;
                }

                if rec1_t < 0.0 {
                    rec1_t = 0.0;
                }

                let mut rang = rand::thread_rng();
                let ray_length = r.dir().length();
                let distance_inside_boundary = (rec2_t - rec1_t) * ray_length;
                let hit_distance =
                    self.neg_inv_density * rang.gen_range(0.0..1.0f32).log(std::f32::consts::E);

                if distance_inside_boundary < hit_distance {
                    return None;
                }

                let t = rec1_t + hit_distance / ray_length;
                Some(HitRecord {
                    t,
                    u: rec1.u,
                    v: rec1.v,
                    p: r.at(t),
                    front_face: true,
                    material: self.phase_function.clone(),
                    normal: Vec3::f32(1.0, 0.0, 0.0),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        self.boundary.bounding_box(time0, time1)
    }
}
pub struct Isotropic {
    albedo: Box<dyn Texture>,
}

impl Isotropic {
    pub fn new(c: Color) -> Self {
        Self {
            albedo: Box::new(SolidColor::new(c)),
        }
    }
}

impl Material for Isotropic {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        Some((
            self.albedo.value(rec.u, rec.v, &rec.p),
            Ray::new(rec.p, Vec3::<f32>::random_in_unit_sphere(), r.time()),
        ))
    }
}
