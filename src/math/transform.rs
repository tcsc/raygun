
use math::{self, Point, Matrix, Vector};
use units::{Angle, Radians, degrees};

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

    pub fn translate(&self, x: f64, y: f64, z: f64) -> Transform {
        let fwd = self.matrix * math::translation_matrix(x, y, z);
        let inv = math::translation_matrix(-x, -y, -z) * self.inverse;
        Transform { matrix: fwd, inverse: inv }
    }

    pub fn rotate(&self, x: Angle<Radians>, y: Angle<Radians>, z: Angle<Radians>)
              -> Transform
    {
        let zero = Angle::<Radians>::new(0.0);
        let mut fwd = self.matrix;
        let mut inv = self.inverse;

        if x != zero {
            fwd = fwd * math::x_rotation_matrix(x);
            inv = math::x_rotation_matrix(-x) * inv;
        }

        if y != zero {
            fwd = fwd * math::y_rotation_matrix(y);
            inv = math::y_rotation_matrix(-y) * inv;
        }

        if z != zero {
            fwd = fwd * math::z_rotation_matrix(z);
            inv = math::z_rotation_matrix(-z) * inv;
        }

        Transform { matrix: fwd, inverse: inv }
    }

    pub fn scale(&self, x: f64, y: f64, z: f64) -> Transform {
        let fwd = self.matrix * math::scaling_matrix(x, y, z);
        let rev = math::scaling_matrix(1.0/x, 1.0/y, 1.0/z) * self.inverse;
        Transform { matrix: fwd, inverse: rev }
    }
}

impl Default for Transform {
    fn default() -> Transform {
        Transform { matrix: math::IDENTITY, inverse: math::IDENTITY }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use math::point;

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