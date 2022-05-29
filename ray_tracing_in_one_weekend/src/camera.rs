use crate::ray::Ray;
use crate::Vec3;

pub struct Camera {
    origin: Vec3<f32>,
    lower_left_corner: Vec3<f32>,
    horizontal: Vec3<f32>,
    vertical: Vec3<f32>,
}

impl Camera {
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}

impl Default for Camera {
    fn default() -> Self {
        let aspect_ration = 16.0 / 9.0;
        let viewport_height = 2.0;
        let viewport_width = aspect_ration * viewport_height;
        let focal_length = 1.0;

        let origin = Vec3::f32(0.0, 0.0, 0.0);
        let horizontal = Vec3::f32(viewport_width, 0.0, 0.0);
        let vertical = Vec3::f32(0.0, viewport_height, 0.0);
        let lower_left_corner =
            origin - horizontal / 2.0 - vertical / 2.0 - Vec3::f32(0.0, 0.0, focal_length);

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }
}
