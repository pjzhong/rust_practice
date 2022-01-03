use std::fmt::Debug;
use std::ops::{Add, Index, IndexMut, Mul, Sub};

#[derive(Debug, PartialEq)]
struct Vec3<T: Debug + PartialEq> {
    x: T,
    y: T,
    z: T,
}

macro_rules! vec3_impl {
    ($($t:ty)+, $n:ident) => ($(

        impl Vec3<$t> {
            fn $n(x: $t, y: $t, z: $t) -> Self {
                Self {
                    x,
                    y,
                    z,
                }
            }

            fn length(&self) -> $t {
                (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
            }

            fn dot_product(&self, v:&Vec3<$t>) -> $t {
                self.x * v.x + self.y * v.y + self.z * v.z
            }

            fn cross(&self,  v:&Vec3<$t>) -> Vec3<$t> {
                return Self {
                   x: self.y * v.z - self.z * v.y,
                   y: self.z * v.x - self.x * v.z,
                   z: self.x * v.y - self.y * v.x
                }
            }

            fn normalize(&mut self) -> &Self {
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

    )+)
}

vec3_impl! { f32,f32 }
vec3_impl! { f64,f64 }

#[derive(Debug, PartialEq)]
struct Matrix44<T: Debug + PartialEq> {
    m: [[T; 4]; 4],
}

macro_rules! matrix_impl {
    ($($t:ty)+, $n:ident) => ($(

        impl Matrix44<$t> {
            fn transpose(&self) -> Matrix44<$t> {
                let mut transp_mat = Matrix44::<$t>::default();

                for i in 0..4 {
                    for j in 0..4 {
                        transp_mat[i][j] = self[j][i];
                    }
                }

                transp_mat
            }
        }

        impl Index<usize> for Matrix44<$t> {
            type Output = [$t;4];

            fn index(&self, index: usize) -> &Self::Output {
                &self.m[index]
            }
        }

        impl IndexMut<usize> for Matrix44<$t> {

            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.m[index]
            }
        }

        impl Default for Matrix44<$t> {
            fn default() -> Self {
                Self {
                    m: [
                        [1., 0., 0., 0.],
                        [0., 1., 0., 0.],
                        [0., 0., 1., 0.],
                        [0., 0., 0., 1.],
                    ]
                }
            }
        }

        impl Mul for Matrix44<$t> {
            type Output = Matrix44<$t>;

            fn mul(self, other: Matrix44<$t>) -> Matrix44<$t> {
                let mut matrix = Matrix44::<$t>::default();

                for i in 0..4 {
                    for j in 0..4 {
                        matrix[i][j] = self[i][0] * other[0][j] +
                                       self[i][1] * other[1][j] +
                                       self[i][2] * other[2][j] +
                                       self[i][3] * other[3][j];
                    }
                }

                matrix
            }
        }

    )+)
}

matrix_impl! { f32,f32 }
matrix_impl! { f64,f64 }

fn main() {
    let v = Vec3::f32(1.0, 1.0, 1.0);
    let v2 = Vec3::f32(1.0, 1.0, 1.0);
    println!("{:?}", v + v2);
}

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

    #[test]
    fn test_matrix_mul_f32() {
        let matrix_one: Matrix44<f32> = Matrix44 {
            m: [
                [2., 0., 0., 0.],
                [0., 2., 0., 0.],
                [0., 0., 2., 0.],
                [0., 0., 0., 2.],
            ],
        };
        let matrix_two = Matrix44 {
            m: [
                [2., 0., 0., 0.],
                [0., 2., 0., 0.],
                [0., 0., 2., 0.],
                [0., 0., 0., 2.],
            ],
        };

        let expect = Matrix44 {
            m: [
                [4., 0., 0., 0.],
                [0., 4., 0., 0.],
                [0., 0., 4., 0.],
                [0., 0., 0., 4.],
            ],
        };

        assert_eq!(expect, matrix_one * matrix_two);
    }

    #[test]
    fn test_matrix_mul_f64() {
        let matrix_one = Matrix44 {
            m: [
                [2., 0., 0., 0.],
                [0., 2., 0., 0.],
                [0., 0., 2., 0.],
                [0., 0., 0., 2.],
            ],
        };
        let matrix_two = Matrix44 {
            m: [
                [2., 0., 0., 0.],
                [0., 2., 0., 0.],
                [0., 0., 2., 0.],
                [0., 0., 0., 2.],
            ],
        };

        let expect = Matrix44 {
            m: [
                [4., 0., 0., 0.],
                [0., 4., 0., 0.],
                [0., 0., 4., 0.],
                [0., 0., 0., 4.],
            ],
        };

        assert_eq!(expect, matrix_one * matrix_two);
    }

    #[test]
    fn test_matrix_transpose() {
        assert_eq!(
            Matrix44::<f32>::default(),
            Matrix44::<f32>::default().transpose()
        );

        let expect = Matrix44::<f64> {
            m: [
                [0., 4., 8., 12.],
                [1., 5., 9., 13.],
                [2., 6., 10., 14.],
                [3., 7., 11., 15.],
            ],
        };

        let matrix = Matrix44::<f64> {
            m: [
                [0., 1., 2., 3.],
                [4., 5., 6., 7.],
                [8., 9., 10., 11.],
                [12., 13., 14., 15.],
            ],
        };

        assert_eq!(expect, matrix.transpose())
    }
}