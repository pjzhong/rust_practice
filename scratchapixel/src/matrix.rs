use std::fmt::Debug;
use std::ops::{Index, IndexMut, Mul};

#[derive(Debug, PartialEq)]
pub struct Matrix44<T: Debug + PartialEq> {
    m: [[T; 4]; 4],
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
