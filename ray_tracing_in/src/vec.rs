use rand::{thread_rng, Rng};
use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Index, Mul, MulAssign, Neg, Sub};

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Vec3<T: Default + Debug + PartialEq + Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
}

macro_rules! vec3_impl {
    ($($t:ty)+, $n:ident) => ($(

        impl Vec3<$t> {
            pub fn $n(x: $t, y: $t, z: $t) -> Self {
                Self {
                    x,
                    y,
                    z,
                }
            }

            pub fn random_in_unit_sphere() -> Self {
                loop {
                    let p = Self::random_range(-1.0, 1.0);
                    if p.length_squared() >= 1.0 {
                        continue;
                    }
                    return p;
                }
            }

            pub fn random_in_unit_disk() -> Self {
                let mut rang = thread_rng();
                loop {
                    let x = rang.gen_range(-1.0..1.0);
                    let y = rang.gen_range(-1.0..1.0);
                    let p = Self::$n(x, y, 0.0);
                    if p.length_squared() >= 1.0 {
                        continue;
                    }
                    return p;
                }
            }

            pub fn random_unit_vecotr() -> Self {
                return Self::random_in_unit_sphere().normalize();
            }

            pub fn random() -> Self {
                Self::random_range(0.0, 1.0)
            }

            pub fn random_range(min: $t, max:$t) -> Self {
                let mut rang = thread_rng();
                Self {
                    x: rang.gen_range(min..max),
                    y: rang.gen_range(min..max),
                    z: rang.gen_range(min..max),
                }
            }

            pub fn length(&self) -> $t {
                (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
            }

            pub fn length_squared(&self) -> $t {
                self.x * self.x + self.y * self.y + self.z * self.z
            }

            pub fn dot_product(&self, v:&Vec3<$t>) -> $t {
                self.x * v.x + self.y * v.y + self.z * v.z
            }

            pub fn cross(&self,  v:&Vec3<$t>) -> Vec3<$t> {
                return Self {
                   x: self.y * v.z - self.z * v.y,
                   y: self.z * v.x - self.x * v.z,
                   z: self.x * v.y - self.y * v.x
                }
            }

            pub fn normalize(mut self) -> Self {
                let len = self.length();
                if len > 0.0 {
                    let inv_len = 1.0 / len;
                    self.x *= inv_len;
                    self.y *= inv_len;
                    self.z *= inv_len;
                }
                self
            }

            pub fn near_zero(&self) -> bool {
                let s = 1e-8;
                return self.x.abs() < s && self.y.abs() < s && self.z.abs() < s;
            }

            pub fn reflect(&self, n: &Self) -> Self {
                return self - &(2.0 * self.dot_product(n) * n) ;
            }

            pub fn refract(&self, n: &Self, etai_over_eta: $t) -> Self {
                let cos_theta = (-self).dot_product(n).min(1.0);
                let r_out_perp = etai_over_eta * (*self + cos_theta * n);
                let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs().sqrt()) * n;
                r_out_perp + r_out_parallel
            }
         }

        impl Add for Vec3<$t> {
            type Output = Vec3<$t>;

            fn add(self, other: Self) -> Self {
                Self {
                    x: self.x + other.x,
                    y: self.y + other.y,
                    z: self.z + other.z,
                }
            }
        }

        impl Sub for Vec3<$t> {
            type Output = Vec3<$t>;

            fn sub(self, other: Self) -> Self {
                Self {
                    x: self.x - other.x,
                    y: self.y - other.y,
                    z: self.z - other.z,
                }
            }
        }

        impl Sub for &Vec3<$t> {
            type Output = Vec3<$t>;

            fn sub(self, other: Self) -> Self::Output {
                Self::Output {
                    x: self.x - other.x,
                    y: self.y - other.y,
                    z: self.z - other.z,
                }
            }
        }

        impl Mul for Vec3<$t> {
            type Output = Vec3<$t>;

            fn mul(self, other: Self) -> Self {
                Self {
                    x: self.x * other.x,
                    y: self.y * other.y,
                    z: self.z * other.z,
                }
            }
        }

        impl Mul<$t> for &Vec3<$t> {
            type Output = Vec3<$t>;

            fn mul(self, rhs: $t) -> Self::Output {
                Self::Output {
                    x: self.x * rhs,
                    y: self.y * rhs,
                    z: self.z * rhs,
                }
            }
        }

        impl Mul<$t> for Vec3<$t> {
            type Output = Vec3<$t>;

            fn mul(self, rhs: $t) -> Self::Output {
                Self {
                    x: self.x * rhs,
                    y: self.y * rhs,
                    z: self.z * rhs,
                }
            }
        }

        impl Mul<Vec3<$t>> for $t {
            type Output = Vec3<$t>;

            fn mul(self, rhs: Vec3<$t>) -> Self::Output {
                rhs * self
            }
        }

        impl Mul<&Vec3<$t>> for $t {
            type Output = Vec3<$t>;

            fn mul(self, rhs: &Vec3<$t>) -> Self::Output {
                rhs * self
            }
        }


        impl Neg for Vec3<$t> {
            type Output = Vec3<$t>;

            fn neg(self) -> Self::Output {
                 Self {
                     x: -self.x,
                     y: -self.y,
                     z: -self.z,
                 }
            }
        }

        impl Neg for &Vec3<$t> {
            type Output = Vec3<$t>;

            fn neg(self) -> Self::Output {
                 Self::Output {
                     x: -self.x,
                     y: -self.y,
                     z: -self.z,
                 }
            }
        }

        impl AddAssign<$t> for Vec3<$t> {
            fn add_assign(&mut self, rhs: $t) {
                self.x += rhs;
                self.y += rhs;
                self.z += rhs;
            }
        }

        impl AddAssign<Vec3<$t>> for Vec3<$t> {
            fn add_assign(&mut self, rhs: Vec3<$t>) {
                self.x += rhs.x;
                self.y += rhs.y;
                self.z += rhs.z;
            }
        }

        impl MulAssign<$t> for Vec3<$t> {
            fn mul_assign(&mut self, rhs: $t) {
                self.x *= rhs;
                self.y *= rhs;
                self.z *= rhs;
            }
        }

        impl Div<$t> for Vec3<$t> {
            type Output = Vec3<$t>;

            fn div(self, rhs: $t) -> Self::Output {
                (1.0 / rhs) * self
            }
        }

        impl Index<usize> for Vec3<$t> {
            type Output = $t;

            fn index(&self, index: usize) -> &Self::Output {
                match index {
                    0 => &self.x,
                    1 => &self.y,
                    2 => &self.z,
                    _ => &0.0,
                }
            }
        }

        impl Sum for Vec3<$t> {
            fn sum<I: Iterator<Item=Self>>(iter: I) -> Self {
                let mut result = Vec3::default();
                for i in iter {
                    result += i;
                }
                result
            }
        }

    )+)
}

vec3_impl! { f32,f32 }
vec3_impl! { f64,f64 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(
            Vec3::f32(2.0, 2.0, 2.0),
            Vec3::f32(1.0, 1.0, 1.0) + Vec3::f32(1.0, 1.0, 1.0)
        );

        assert_eq!(
            Vec3::f64(2.0, 2.0, 2.0),
            Vec3::f64(1.0, 1.0, 1.0) + Vec3::f64(1.0, 1.0, 1.0)
        );

        let mut v = Vec3::f32(3.0, 0.0, 8.0);
        v += 2.0;
        assert_eq!(Vec3::f32(5.0, 2.0, 10.0), v);
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            Vec3::f32(0.0, 0.0, 0.0),
            Vec3::f32(1.0, 1.0, 1.0) - Vec3::f32(1.0, 1.0, 1.0)
        );

        assert_eq!(
            Vec3::f64(0.0, 0.0, 0.0),
            Vec3::f64(1.0, 1.0, 1.0) - Vec3::f64(1.0, 1.0, 1.0)
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            Vec3::f32(4.0, 1.0, 9.0),
            Vec3::f32(2.0, 1.0, 3.0) * Vec3::f32(2.0, 1.0, 3.0)
        );

        assert_eq!(
            Vec3::f64(4.0, 1.0, 9.0),
            Vec3::f64(2.0, 1.0, 3.0) * Vec3::f64(2.0, 1.0, 3.0)
        );

        let mut v = Vec3::f32(3.0, 0.0, 8.0);
        v *= 2.0;
        assert_eq!(Vec3::f32(6.0, 0.0, 16.0), v);
    }

    #[test]
    fn test_length() {
        assert_eq!(3.0_f32.sqrt(), Vec3::f32(1.0, 1.0, 1.0).length());

        assert_eq!(27_f64.sqrt(), Vec3::f64(3.0, 3.0, 3.0).length());
    }
}
