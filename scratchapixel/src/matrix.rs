use std::fmt::Debug;
use std::ops::{Index, IndexMut, Mul};

use crate::vec::Vec3;

#[derive(Debug, PartialEq, Clone)]
pub struct Matrix44<T: Debug + PartialEq + Clone> {
    pub m: [[T; 4]; 4],
}

macro_rules! matrix_impl {
    ($($t:ty)+, $n:ident) => ($(

        impl Matrix44<$t> {
            pub fn transpose(&self) -> Matrix44<$t> {
                let mut transp_mat = Matrix44::<$t>::default();

                for i in 0..4 {
                    for j in 0..4 {
                        transp_mat[i][j] = self[j][i];
                    }
                }

                transp_mat
            }

            pub fn mult_vec_matrix(&self, src: &Vec3<$t>, dst: &mut Vec3<$t>) {
                let x = src.x * self[0][0] + src.y * self[1][0] + src.z * self[2][0] + self[3][0];
                let y = src.x * self[0][1] + src.y * self[1][1] + src.z * self[2][1] + self[3][1];
                let z = src.x * self[0][2] + src.y * self[1][2] + src.z * self[2][2] + self[3][2];
                let w = src.x * self[0][3] + src.y * self[1][3] + src.z * self[2][3] + self[3][3];

                dst.x = x / w;
                dst.y = y / w;
                dst.z = z / w;
            }

            pub fn inverse(&self) -> Self {
                let mut s = Matrix44::<$t>::default();
                let mut t = self.clone();
                for i in 0..3 {
                    let mut pivot = i;

                    let mut pivotsize = t[i][i];

                    if pivotsize < 0.0 {
                        pivotsize = -pivotsize;
                    }

                    for j in i + 1..4 {
                        let mut tmp = t[j][i];

                        if tmp < 0.0 {
                            tmp = -tmp;
                            if tmp > pivotsize {
                                pivot = j;
                                pivotsize = tmp;
                            }
                        }
                    }

                    if pivotsize == 0.0 {
                        return Matrix44::<$t>::default();
                    }

                    if pivot != i {
                        for j in 0..4 {
                            let mut tmp = t[i][j];
                            t[i][j] = t[pivot][j];
                            t[pivot][j] = tmp;

                            tmp = s[i][j];
                            s[i][j] =  s[pivot][j];
                            s[pivot][j] = tmp;
                        }
                    }

                    for j in i + 1..4 {
                        let f = t[j][i] / t[i][i];

                        for k in 0..4 {
                            t[j][k] -= f * t[i][k];
                            s[j][k] -= f * s[i][k];
                        }
                    }
                }

                for i in (0..=3).rev() {
                    let mut f = t[i][i];
                    if f == 0.0 {
                       return Matrix44::<$t>::default();
                    }

                    for j in 0..4 {
                        t[i][j] /= f;
                        s[i][j] /= f;
                    }

                    for j in 0..i {
                        f = t[j][i];

                        for k in 0..4 {
                            t[j][k] -= f * t[i][k];
                            s[j][k] -= f * s[i][k];
                        }
                    }
                }

               return s;
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

#[cfg(test)]
mod tests {
    use super::*;

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
