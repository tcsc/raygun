//! Implements matrix operations suited for 3D graphics work.

use std::fmt;
use std::cmp;
use std::ops;

use math::Vector;
use units::{Angle, Radians};

macro_rules! idx {
    ($i:expr, $j:expr) => { (($i * 4) + $j) }
}

///
/// Defines a 4x4 matrix for manipulating 3D vectors & points in homogeneous
/// co-ordinates. Matrix values are generally immutable, and most operations
/// on them generate a new matrix.
///
#[derive(Copy, Clone)]
pub struct Matrix([f64; 16]);

///
/// The identity matrix
///
pub static IDENTITY: Matrix = Matrix([1.0, 0.0, 0.0, 0.0,
                                      0.0, 1.0, 0.0, 0.0,
                                      0.0, 0.0, 1.0, 0.0,
                                      0.0, 0.0, 0.0, 1.0]);

///
/// An all-zero matrix
///
pub static ZERO: Matrix = Matrix([0.0, 0.0, 0.0, 0.0,
                                  0.0, 0.0, 0.0, 0.0,
                                  0.0, 0.0, 0.0, 0.0,
                                  0.0, 0.0, 0.0, 0.0]);

pub fn translation_matrix(x: f64, y: f64, z: f64) -> Matrix {
    Matrix([
        1.0, 0.0, 0.0,   x,
        0.0, 1.0, 0.0,   y,
        0.0, 0.0, 1.0,   z,
        0.0, 0.0, 0.0, 1.0
    ])
}

pub fn scaling_matrix(x: f64, y: f64, z: f64) -> Matrix {
    Matrix([
          x, 0.0, 0.0, 0.0,
        0.0,   y, 0.0, 0.0,
        0.0, 0.0,   z, 0.0,
        0.0, 0.0, 0.0, 1.0
    ])
}

pub fn x_rotation_matrix(theta: Angle<Radians>) -> Matrix {
    let s = theta.sin();
    let c = theta.cos();
    Matrix([
        1.0, 0.0, 0.0, 0.0,
        0.0,   c,  -s, 0.0,
        0.0,   s,   c, 0.0,
        0.0, 0.0, 0.0, 1.0
    ])
}

pub fn y_rotation_matrix(theta: Angle<Radians>) -> Matrix {
    let s = theta.sin();
    let c = theta.cos();
    Matrix([
          c, 0.0,   s, 0.0,
        0.0, 1.0, 0.0, 0.0,
         -s, 0.0,   c, 0.0,
        0.0, 0.0, 0.0, 1.0
    ])
}

pub fn z_rotation_matrix(theta: Angle<Radians>) -> Matrix {
    let s = theta.sin();
    let c = theta.cos();
    Matrix([
          c,  -s, 0.0, 0.0,
          s,   c, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    ])
}

impl Matrix {
    pub fn transpose(&self) -> Matrix {
        let &Matrix(ref m) = self;
        let Matrix(mut result) = ZERO;

        for j in 0usize..4usize {
            for i in 0usize..4usize {
                result[idx!(i, j)] = m[idx!(j, i)]
            }
        }
        return Matrix(result);
    }
}

impl Default for Matrix {
    fn default() -> Matrix {
        ZERO
    }
}

///
/// Debug formatting
///
impl fmt::Debug for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Matrix(ref values) = self;

        for n in 0..4 {
            let row = n * 4;
            match write!(f,
                         "({}, {}, {}, {})",
                         values[row],
                         values[row + 1],
                         values[row + 2],
                         values[row + 3]) {
                e @ Err(_) => return e,
                _ => (),
            }
        }
        Ok(())
    }
}

///
/// Implements equality tests for matrices
///
impl cmp::PartialEq for Matrix {
    fn eq(&self, other: &Matrix) -> bool {
        let &Matrix(ref a) = self;
        let &Matrix(ref b) = other;
        a == b
    }

    fn ne(&self, other: &Matrix) -> bool {
        let &Matrix(ref a) = self;
        let &Matrix(ref b) = other;
        a != b
    }
}

impl Eq for Matrix {}

impl ops::Index<(usize, usize)> for Matrix {
    type Output = f64;

    fn index<'a>(&'a self, idx: (usize, usize)) -> &'a f64 {
        let &Matrix(ref values) = self;
        let (i, j) = idx;
        &values[(j * 4) + i]
    }
}

///
/// Implements m[(i,j)] = x.
//
impl ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut<'a>(&'a mut self, idx: (usize, usize)) -> &'a mut f64 {
        let Matrix(ref mut values) = *self;
        let (i, j) = idx;
        &mut values[(j * 4) + i]
    }
}

///
/// Takes the dot product of a given row and column of two matrices, as part of
/// matrix multiplication.
///
#[inline]
fn row_col_dot_product(lhs: &Matrix, rhs: &Matrix, i: usize, j: usize) -> f64 {
    let &Matrix(ref a) = lhs;
    let &Matrix(ref b) = rhs;

    (a[idx!(0,j)] * b[idx!(i,0)]) +
    (a[idx!(1,j)] * b[idx!(i,1)]) +
    (a[idx!(2,j)] * b[idx!(i,2)]) +
    (a[idx!(3,j)] * b[idx!(i,3)])
}

///
/// Implements proper matrix multiplication for our 4x4 matrices
///
impl ops::Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Matrix {
        let Matrix(mut result) = ZERO;
        for i in 0usize..4usize {
            for j in 0usize..4usize {
                result[idx!(i,j)] = row_col_dot_product(&self, &rhs, i, j)
            }
        }
        return Matrix(result);
    }
}

impl ops::Mul<Vector> for Matrix {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Vector {
        let Matrix(ref m) = self;
        let x = (m[idx!(0,0)] * rhs.x) +
                (m[idx!(0,1)] * rhs.y) +
                (m[idx!(0,2)] * rhs.z) +
                 m[idx!(0,3)];

        let y = (m[idx!(1,0)] * rhs.x) +
                (m[idx!(1,1)] * rhs.y) +
                (m[idx!(1,2)] * rhs.z) +
                 m[idx!(1,3)];

        let z = (m[idx!(2,0)] * rhs.x) +
                (m[idx!(2,1)] * rhs.y) +
                (m[idx!(2,2)] * rhs.z) +
                 m[idx!(2,3)];

        let w = m[idx!(3,3)];
        let inv_w = 1.0 / w;

        Vector::new(x * inv_w, y * inv_w, z * inv_w)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use units::degrees;
    use float_cmp::{Ulps, ApproxEqUlps};

    #[test]
    fn matrix_transpose() -> () {
        let mut m = IDENTITY;
        m[(0, 3)] = 42.0;

        let mt = m.transpose();

        assert_eq!(mt[(0, 3)], 0.0);
        assert_eq!(mt[(3, 0)], 42.0);
    }

    #[test]
    fn matrix_transpose_reflexivity() -> () {
        assert_eq!(IDENTITY, IDENTITY.transpose().transpose())
    }

    #[test]
    fn matrix_default() -> () {
        let m: Matrix = Default::default();
        assert_eq!(m, ZERO)
    }

    #[test]
    fn matrix_index_mut() {
        let mut m = IDENTITY;
        assert_eq!(m[(1, 2)], 0.0);
        m[(1, 2)] = 42.0;
        assert_eq!(m[(1, 2)], 42.0);
        assert_eq!(m[(2, 1)], 0.0);
    }

    #[test]
    fn matrix_multiply_identity() {
        let m = IDENTITY * IDENTITY;
        assert_eq!(m, IDENTITY);
    }

    #[test]
    fn rotation_about_x_matrix() {
        let m = x_rotation_matrix(degrees(90.0).radians());
        let expected = Matrix([1.0, 0.0,  0.0, 0.0,
                               0.0, 0.0, -1.0, 0.0,
                               0.0, 1.0,  0.0, 0.0,
                               0.0, 0.0, 0.0, 1.0]);

        for i in 0..4 {
            for j in 0..4 {
                let actual = m[(i, j)];
                let exp = expected[(i, j)];
                if exp == 0.0 {
                    assert!(actual < 1e-16,
                            "[{},{}]: expected {}, got {}",
                            i,
                            j,
                            exp,
                            actual);
                } else {
                    assert!(exp.approx_eq_ulps(&actual, 100),
                            "[{},{}]: expected {}, got {}",
                            i,
                            j,
                            exp,
                            actual);
                }
            }
        }
    }
}
