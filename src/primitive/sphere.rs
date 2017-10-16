use std::cmp;
use math::{self, Point, Vector, point};

use ray::Ray;
use primitive::{AxisAlignedBox, Primitive};

///
/// A Sphere primitive
///
#[derive(Debug)]
pub struct Sphere {
    pub centre: Point,
    pub radius: f64
}

impl Sphere {
    pub fn new(loc: Point, radius: f64) -> Sphere {
        Sphere{ centre: loc, radius: radius }
    }
}

impl Default for Sphere {
    fn default() -> Sphere {
        Sphere {
            centre: point(0.0, 0.0, 0.0),
            radius: 1.0
        }
    }
}

/// Implements a naive, bit-pattern-equality test for a sphere object
impl cmp::PartialEq for Sphere {
    fn eq(&self, other: &Sphere) -> bool {
        self.centre == other.centre && self.radius == other.radius
    }

    fn ne(&self, other: &Sphere) -> bool {
        self.centre != other.centre || self.radius != other.radius
    }
}

impl cmp::Eq for Sphere {}

impl Primitive for Sphere {
    fn intersects(&self, r: Ray) -> Option<f64> {
        let dist = Vector::between(r.src, self.centre);
        let b = r.dir.dot(dist);
        match (b * b) - dist.dot(dist) + (self.radius * self.radius) {
            n if n < 0.0 => { None }
            d2 => {
                let d = d2.sqrt();
                let t1 = b - d;
                let t2 = b + d;
                if t2 > 0.0 {
                    Some( if t1 > 0.0 { t1 } else { t2 } )
                }
                else {
                    None
                }
            }
        }
    }

    fn bounding_box(&self) -> AxisAlignedBox {
        let (min_x, max_x) = math::sort(self.centre.x - self.radius,
                                        self.centre.x + self.radius);
        let (min_y, max_y) = math::sort(self.centre.y - self.radius,
                                        self.centre.y + self.radius);
        let (min_z, max_z) = math::sort(self.centre.z - self.radius,
                                        self.centre.z + self.radius);
        AxisAlignedBox {
            lower: point(min_x, min_y, min_z),
            upper: point(max_x, max_y, max_z)
        }
    }

    fn normal(&self, pt: Point) -> Vector {
        (pt - self.centre).normalize()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use math::*;
    use ray::*;
    use primitive::primitive::*;

    #[test]
    fn default() {
        let s : Sphere = Default::default();
        let expected = Sphere {
            centre: point(0.0, 0.0, 0.0),
            radius: 1.0
        };
        assert_eq!(s, expected)
    }

    #[test]
    fn intersecting_ray_intersects() {
        let s = Sphere::default();
        let r = Ray::new(point(0.0, 0.0, -10.0), vector(0.0, 0.0, 1.0));
        if let Some(x) = s.intersects(r) {
            assert!(x - 90.0 < 0.0000001)
        }
        else {
            panic!("Did not intersect")
        }
    }

    #[test]
    fn non_intersecting_ray_doesnt() {
        let s = Sphere::default();
        let r = Ray::new(point(0.0, 10.0, 0.0), vector(0.0, 0.0, 1.0));
        if let Some(_) = s.intersects(r) {
            panic!("Ray intersects")
        }
    }

    #[test]
    fn cardinal_normals_are_as_expected() {
        let s = Sphere::default();

        let p1 = point(1.0, 0.0, 0.0);
        let v1 = vector(1.0, 0.0, 0.0);
        let n1 = s.normal(p1);
        assert!(n1.approx_eq(v1), "Expected {:?}, got {:?}", v1, n1);

        let p2 = point(0.0, 1.0, 0.0);
        let v2 = vector(0.0, 1.0, 0.0);
        let n2 = s.normal(p2);
        assert!(n2.approx_eq(v2), "Expected {:?}, got {:?}", v2, n2);

        let p3 = point(0.0, 0.0, 1.0);
        let v3 = vector(0.0, 0.0, 1.0);
        let n3 = s.normal(p3);
        assert!(n3.approx_eq(v3), "Expected {:?}, got {:?}", v3, n3);
    }
}