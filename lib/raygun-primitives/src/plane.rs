use crate::{AxisAlignedBox, Primitive};

use raygun_math::{
    Point, Vector, Ray, point, vector,
};

#[derive(Debug)]
pub struct Plane {
    pub normal: Vector,
    pub offset: f64
}

impl Primitive for Plane {
    fn intersects(&self, r: Ray) -> Option<f64> {
        let n = self.offset - r.src.dot(self.normal);
        let d = r.dir.dot(self.normal);
        match n / d {
            a if a > 0.0 => {
                Some(a)
            },
            _ => None
        }
    }

    fn normal(&self, _pt: Point) -> Vector {
        self.normal
    }

    fn bounding_box(&self) -> AxisAlignedBox {
        use std::f64::{INFINITY, NEG_INFINITY};

        AxisAlignedBox {
            lower: point(NEG_INFINITY, NEG_INFINITY, NEG_INFINITY),
            upper: point(INFINITY,INFINITY,INFINITY)
        }
    }
}

impl Default for Plane {
    fn default() -> Plane {
        Plane {
            normal: vector(0.0, 1.0, 0.0),
            offset: 0.0
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use raygun_math::{point, vector};
    use std::f64::consts::SQRT_2;
    use float_cmp::approx_eq;

    #[test]
    fn intersecting_ray_intersects() {
        let r = Ray::new(point(0.0, 1.0, 0.0),
                         vector(0.0, -1.0, 1.0).normalize());
        let p = Plane {
            normal: vector(0.0, 1.0, 0.0),
            offset: 0.0
        };

        let value = p.intersects(r).unwrap();
        assert!(approx_eq!(f64, value, SQRT_2, ulps = 5),
                "Expected {}, got {}", SQRT_2, value);
    }

    #[test]
    fn non_intersecting_ray_does_not() {
        let r = Ray::new(point(0.0, 1.0, 0.0),
                         vector(0.0, 0.0, 1.0).normalize());
        let p = Plane {
            normal: vector(0.0, 1.0, 0.0),
            offset: 0.0
        };

        assert!(p.intersects(r).is_none());
    }

    #[test]
    fn normal() {
        let p = Plane {
            normal: vector(0.0, 1.0, 0.0),
            offset: 0.0
        };

        assert_eq!(p.normal(point(100.0, 0.0, 100.0)),
                   p.normal);
    }
}
