use crate::Point;
use rand::prelude::SliceRandom;
use rand::Rng;

pub struct Perlin {
    ran_float: Vec<f32>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Perlin {
    const POINT_COUNT: usize = 256;

    pub fn new() -> Self {
        let mut ran_float = vec![];
        let mut rang = rand::thread_rng();
        for _ in 0..Perlin::POINT_COUNT {
            ran_float.push(rang.gen_range(0.0..1.0));
        }

        Self {
            ran_float,
            perm_x: Perlin::perlin_generate_perm(),
            perm_y: Perlin::perlin_generate_perm(),
            perm_z: Perlin::perlin_generate_perm(),
        }
    }

    pub fn noise(&self, p: &Point) -> f32 {
        let i = ((4.0 * p.x) as i32 & 255) as usize;
        let j = ((4.0 * p.y) as i32 & 255) as usize;
        let k = ((4.0 * p.z) as i32 & 255) as usize;

        self.ran_float[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
    }

    fn perlin_generate_perm() -> Vec<usize> {
        let mut result = vec![];
        for i in 0..Perlin::POINT_COUNT {
            result.push(i);
        }

        result.shuffle(&mut rand::thread_rng());
        result
    }
}

impl Default for Perlin {
    fn default() -> Self {
        Perlin::new()
    }
}
