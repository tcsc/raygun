use std::cmp;
use math::*;
use ray::Ray;
use primitive::primitive::Primitive;

///
/// A Sphere primitive
///
#[derive(Debug)]
pub struct Sphere {
    centre: Point,
    radius: f64
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
}