use crate::vec::Vec3;

#[derive(Debug, Default)]
pub struct Ray {
    origin: Vec3<f32>,
    dir: Vec3<f32>,
}

impl Ray {
    pub fn new(origin: Vec3<f32>, dir: Vec3<f32>) -> Self {
        Self { origin, dir }
    }

    pub fn origin(&self) -> &Vec3<f32> {
        &self.origin
    }

    pub fn dir(&self) -> &Vec3<f32> {
        &self.dir
    }

    pub fn at(&self, t: f32) -> Vec3<f32> {
        self.origin + t * self.dir
    }
}
