use crate::Point;
use rand::prelude::SliceRandom;

pub struct Perlin {
    ran_float: Vec<Point>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Perlin {
    const POINT_COUNT: usize = 256;

    pub fn new() -> Self {
        let mut ran_float = vec![];
        for _ in 0..Perlin::POINT_COUNT {
            ran_float.push(Point::random_range(-1.0, 1.0));
        }

        Self {
            ran_float,
            perm_x: Perlin::perlin_generate_perm(),
            perm_y: Perlin::perlin_generate_perm(),
            perm_z: Perlin::perlin_generate_perm(),
        }
    }

    fn perlin_generate_perm() -> Vec<usize> {
        let mut result = vec![];
        for i in 0..Perlin::POINT_COUNT {
            result.push(i);
        }

        result.shuffle(&mut rand::thread_rng());
        result
    }

    pub fn noise(&self, p: &Point) -> f32 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;

        let mut c = [
            [
                [Point::default(), Point::default()],
                [Point::default(), Point::default()],
            ],
            [
                [Point::default(), Point::default()],
                [Point::default(), Point::default()],
            ],
        ];

        for (di, di_c) in c.iter_mut().enumerate() {
            for (dj, dj_c) in di_c.iter_mut().enumerate() {
                for (dk, dk_c) in dj_c.iter_mut().enumerate() {
                    let xi = ((i + di as i32) & 255) as usize;
                    let yj = ((j + dj as i32) & 255) as usize;
                    let yk = ((k + dk as i32) & 255) as usize;
                    *dk_c = self.ran_float[self.perm_x[xi] ^ self.perm_y[yj] ^ self.perm_z[yk]]
                }
            }
        }

        Perlin::trilinear_interp(&c, u, v, w)
    }

    fn trilinear_interp(c: &[[[Point; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
        let u = u * u * (3.0 - 2.0 * u);
        let v = v * v * (3.0 - 2.0 * v);
        let w = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0;

        for (i, ic) in c.iter().enumerate() {
            for (j, jc) in ic.iter().enumerate() {
                for (k, element) in jc.iter().enumerate() {
                    let fi = i as f32;
                    let fj = j as f32;
                    let fk = k as f32;
                    let weight_v = Point::f32(u - fi, v - fj, w - fk);
                    accum += (fi * u + (1.0 - fi) * (1.0 - u))
                        * (fj * v + (1.0 - fj) * (1.0 - v))
                        * (fk * w + (1.0 - fk) * (1.0 - w))
                        * element.dot_product(&weight_v);
                }
            }
        }

        accum
    }

    pub fn turb(&self, p: &Point, depth: usize) -> f32 {
        let mut accum = 0.0;
        let mut temp_p = *p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }
}

impl Default for Perlin {
    fn default() -> Self {
        Perlin::new()
    }
}
