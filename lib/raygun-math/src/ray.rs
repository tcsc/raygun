use crate::{Matrix, Point, Vector};

///
/// Represents a ray through the scene, starting at `src` and heading along
/// `dir`. The vector is always normalised.
///
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Ray {
    pub src: Point,
    pub dir: Vector,
}

impl Ray {
    /// Initialises a new `Ray` instance, normalising the supplied vector
    /// during construction.
    pub fn new(src: Point, dir: Vector) -> Ray {
        Ray {
            src: src,
            dir: dir.normalize(),
        }
    }

    /// Calculates the point `len` units along the ray
    pub fn extend(&self, len: f64) -> Point {
        self.src + (self.dir * len)
    }

    /// Reflect the ray vector through the surface normal.
    /// http://www.3dkingdoms.com/weekly/weekly.php?a=2
    pub fn reflect(&self, normal: Vector, surface: Point) -> Ray {
        let dir = (-2.0 * self.dir.dot(normal) * normal) + self.dir;
        Ray {
            src: surface,
            dir: dir.normalize(),
        }
    }

    pub fn transform(&self, t: &Matrix) -> Ray {
        let s = t * self.src;
        let d = self.dir.transform(t).normalize();
        Ray::new(s, d)
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Ray {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
        Ray::new(Point::arbitrary(g), Vector::arbitrary(g))
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use quickcheck::{quickcheck, TestResult};
    use quickcheck_macros::quickcheck as quickcheck_test;

    #[quickcheck_test]
    fn construction(p: Point, v: Vector) -> bool {
        let r = Ray::new(p, v);

        (r.src == p) && (r.dir == v.normalize())
    }

    #[quickcheck_test]
    fn extention(r: Ray, f: f64) -> bool {
        let pt = r.extend(f);
        let expected = point(
            r.src.x + (f * r.dir.x),
            r.src.y + (f * r.dir.y),
            r.src.z + (f * r.dir.z),
        );
        pt.approx_eq(expected)
    }

    #[test]
    fn reflection() {
        let origin = point(0.0, 0.0, 0.0);
        let inbound = Ray {
            src: point(0.0, 1.0, -1.0),
            dir: vector(0.0, -1.0, 1.0).normalize(),
        };

        let outbound = inbound.reflect(vector(0.0, 1.0, 0.0), origin);

        assert_eq!(outbound.src, origin);
        let expected = vector(0.0, 1.0, 1.0).normalize();
        assert!(outbound.dir.approx_eq(expected));
    }

    #[test]
    fn translation() {
        let m = IDENTITY * translation_matrix(0.0, 1.0, 0.0);
        let r = Ray::new(point(0.0, 0.0, -1.0), point(0.0, 0.0, 1.0));
        let rt = r.transform(&m);

        let expected_src = point(0.0, 1.0, -1.0);
        assert!(
            rt.src.approx_eq(expected_src),
            "Expected src {:?}, got {:?}",
            expected_src,
            rt.src
        );

        let expected_dir = vector(0.0, 0.0, 1.0);
        assert!(
            rt.dir.approx_eq(expected_dir),
            "Expected dir {:?}, got {:?}",
            expected_dir,
            rt.dir
        );
    }
}
