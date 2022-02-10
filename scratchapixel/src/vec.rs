use std::fmt::Debug;
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vec3<T: Debug + PartialEq + Copy> {
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

            pub fn length(&self) -> $t {
                (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
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

        impl Default for Vec3<$t> {
            fn default() -> Self {
                Self {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }
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
    }

    #[test]
    fn test_length() {
        assert_eq!(3.0_f32.sqrt(), Vec3::f32(1.0, 1.0, 1.0).length());

        assert_eq!(27_f64.sqrt(), Vec3::f64(3.0, 3.0, 3.0).length());
    }
}
