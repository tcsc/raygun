use super::{
    Point,
    Matrix,
    Vector,
    translation_matrix,
};

use crate::units::{Angle, Radians, degrees};

///
/// A Transform consists of a pair of matrices describing an affine
/// transformation and its inverse.
///
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Transform {
    pub matrix: Matrix,
    pub inverse: Matrix,
}

impl Transform {
    pub fn identity() -> Transform {
        Transform::default()
    }

    pub fn for_translation(x: f64, y: f64, z: f64) -> Transform {
        let fwd = translation_matrix(x, y, z);
        let rev = translation_matrix(-x, -y, -z);
        Transform { matrix: fwd, inverse: rev }
    }

    pub fn translate(&self, x: f64, y: f64, z: f64) -> Transform {
        self.apply(&Transform::for_translation(x, y, z))
    }

    pub fn for_rotation(x: Angle<Radians>, y: Angle<Radians>, z: Angle<Radians>)
        -> Transform
    {
        let zero = Angle::<Radians>::new(0.0);
        let mut fwd = super::IDENTITY;
        let mut inv = super::IDENTITY;

        if x != zero {
            fwd = fwd * super::x_rotation_matrix(x);
            inv = super::x_rotation_matrix(-x) * inv;
        }

        if y != zero {
            fwd = fwd * super::y_rotation_matrix(y);
            inv = super::y_rotation_matrix(-y) * inv;
        }

        if z != zero {
            fwd = fwd * super::z_rotation_matrix(z);
            inv = super::z_rotation_matrix(-z) * inv;
        }

        Transform { matrix: fwd, inverse: inv }
    }

    pub fn rotate(&self, x: Angle<Radians>, y: Angle<Radians>, z: Angle<Radians>)
        -> Transform
    {
        self.apply(&Transform::for_rotation(x, y, z))
    }

    pub fn for_scale(x: f64, y: f64, z: f64) -> Transform {
        let fwd = super::scaling_matrix(x, y, z);
        let rev = super::scaling_matrix(1.0/x, 1.0/y, 1.0/z);
        Transform { matrix: fwd, inverse: rev }
    }

    pub fn scale(&self, x: f64, y: f64, z: f64) -> Transform {
        self.apply(&Transform::for_scale(x, y, z))
    }

    pub fn apply(&self, other: &Transform) -> Transform {
        let fwd = self.matrix * other.matrix;
        let rev = other.inverse * self.inverse;
        Transform { matrix: fwd, inverse: rev }
    }
}

impl Default for Transform {
    fn default() -> Transform {
        Transform { matrix: super::IDENTITY, inverse: super::IDENTITY }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::math::point;

    #[test]
    fn rotation() {
        let test_cases = [
            ((90.0,  0.0,  0.0), point( 0.0, -1.0,  0.0)),
            (( 0.0, 90.0,  0.0), point( 1.0,  0.0,  0.0)),
            (( 0.0,  0.0, 90.0), point( 0.0,  0.0,  1.0)),
            ((90.0,  0.0, 90.0), point( 1.0,  0.0,  0.0)),
        ];

        let p = point(0.0, 0.0, 1.0);

        for &((rx, ry, rz), expected) in test_cases.iter() {
            let t = Transform::default().rotate(degrees(rx).radians(),
                                                degrees(ry).radians(),
                                                degrees(rz).radians());

            let new_p = t.matrix * p;

            assert!(new_p.approx_eq(expected),
                    "Rotation: (x: {}, y: {}, z: {}), expected {:?}, got {:?}",
                    rx, ry, rz, expected, new_p);

            let inv_p = t.inverse * new_p;
            assert!(inv_p.approx_eq(p),
                    "Inverse rotation: (x: {}, y: {}, z: {}), expected {:?}, got {:?}",
                    rx, ry, rz, p, inv_p);
        }
    }

    #[test]
    fn translation() {
        let test_cases = [
            ((1.0, 0.0, 0.0), point(1.0, 0.0, 0.0)),
            ((0.0, 1.0, 0.0), point(0.0, 1.0, 0.0)),
            ((0.0, 0.0, 1.0), point(0.0, 0.0, 1.0)),
            ((1.0, 2.0, 3.0), point(1.0, 2.0, 3.0)),
        ];

        let p = point(0.0, 0.0, 0.0);

        for &((tx, ty, tz), expected) in test_cases.iter() {
            let t = Transform::default().translate(tx, ty, tz);

            let new_p = t.matrix * p;

            assert!(new_p.approx_eq(expected),
                    "Translation: (x: {}, y: {}, z: {}), expected {:?}, got {:?}",
                    tx, ty, tz, expected, new_p);

            let inv_p = t.inverse * new_p;
            assert!(inv_p.approx_eq(p),
                    "Inverse translation: (x: {}, y: {}, z: {}), expected {:?}, got {:?}",
                    tx, ty, tz, p, inv_p);
        }
    }

    #[test]
    fn scale() {
        let test_cases = [
            ((5.0, 1.0, 1.0), point(5.0, 1.0, 1.0)),
            ((1.0, 5.0, 1.0), point(1.0, 5.0, 1.0)),
            ((1.0, 1.0, 5.0), point(1.0, 1.0, 5.0)),
            ((2.0, 3.0, 4.0), point(2.0, 3.0, 4.0)),
        ];

        let p = point(1.0, 1.0, 1.0);

        for &((sx, sy, sz), expected) in test_cases.iter() {
            let t = Transform::default().scale(sx, sy, sz);

            let new_p = t.matrix * p;

            assert!(new_p.approx_eq(expected),
                    "Scale: (x: {}, y: {}, z: {}), expected {:?}, got {:?}",
                    sx, sy, sz, expected, new_p);

            let inv_p = t.inverse * new_p;
            assert!(inv_p.approx_eq(p),
                    "Inverse scale: (x: {}, y: {}, z: {}), expected {:?}, got {:?}",
                    sx, sy, sz, p, inv_p);
        }
    }

    #[test]
    fn stacked_transforms() {
        let p = point(1.0, 1.0, 1.0);
        let t = Transform::default().rotate(degrees(90.0).radians(),
                                            degrees(0.0).radians(),
                                            degrees(0.0).radians())
                                    .translate(2.0, 3.5, 4.0)
                                    .scale(2.0, 1.0, 1.0);
        let expected = point(6.0, 2.5, 5.0);
        let new_p = t.matrix * p;
        assert!(new_p.approx_eq(expected),
                "Stacked: expected {:?}, got {:?}", expected, new_p);

        let inv_p = t.inverse * new_p;
        assert!(inv_p.approx_eq(p),
                "Stacked: expected {:?}, got {:?}", p, inv_p);
    }
}