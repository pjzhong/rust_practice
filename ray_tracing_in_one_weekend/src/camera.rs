use crate::ray::Ray;
use crate::Vec3;

pub struct Camera {
    origin: Vec3<f32>,
    lower_left_corner: Vec3<f32>,
    horizontal: Vec3<f32>,
    vertical: Vec3<f32>,
    u: Vec3<f32>,
    v: Vec3<f32>,
    lens_radius: f32,
}

impl Camera {
    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        let rd = self.lens_radius * Vec3::<f32>::random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}

impl Camera {
    pub fn new(
        look_from: Vec3<f32>,
        look_at: Vec3<f32>,
        vup: Vec3<f32>,
        vfov: f32,
        aspect_ration: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ration * viewport_height;

        let w = (look_from - look_at).normalize();
        let u = vup.cross(&w).normalize();
        let v = w.cross(&u);

        let origin = look_from;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        let lens_radius = aperture / 2.0;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius,
        }
    }
}
