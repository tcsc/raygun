use math::{Point, Vector};
use ray::Ray;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxisAlignedBox {
    pub lower: Point,
    pub upper: Point
}

impl AxisAlignedBox {
    pub fn union(&self, other: &AxisAlignedBox) -> AxisAlignedBox {

        let lower = Point {
            x: f64::min(self.lower.x, other.lower.x),
            y: f64::min(self.lower.y, other.lower.y),
            z: f64::min(self.lower.z, other.lower.z)
        };

        let upper = Point {
            x: f64::max(self.upper.x, other.upper.x),
            y: f64::max(self.upper.y, other.upper.y),
            z: f64::max(self.upper.z, other.upper.z)
        };

        AxisAlignedBox { lower, upper}
    }

    pub fn intersects(&self, r: &Ray) -> Option<f64> {
        let t_lower_x = (self.lower.x - r.src.x) / r.dir.x;
        let t_upper_x = (self.upper.x - r.src.x) / r.dir.x;
        let t_lower_y = (self.lower.y - r.src.y) / r.dir.y;
        let t_upper_y = (self.upper.y - r.src.y) / r.dir.y;
        let t_lower_z = (self.lower.z - r.src.z) / r.dir.z;
        let t_upper_z = (self.upper.z - r.src.z) / r.dir.z;

        let t_min = f64::max(f64::max(f64::min(t_lower_x, t_upper_x),
                                      f64::min(t_lower_y, t_upper_y)),
                             f64::min(t_lower_z, t_upper_z));

        let t_max = f64::min(f64::min(f64::max(t_lower_x, t_upper_x),
                                      f64::max(t_lower_y, t_upper_y)),
                             f64::max(t_lower_z, t_upper_z));

        if t_max < 0.0 {
            // ray intersects box if extended infinitely, but the whole box
            // is behind the ray origin, which doesn't count
            //error!("Should never happen");
            None
        } else if t_min > t_max {
            // Ray does not intersect box
            None
        } else {
            Some(t_min)
        }
    }
}

impl Default for AxisAlignedBox {
    fn default() -> AxisAlignedBox {
        AxisAlignedBox {
            lower: Point::default(),
            upper: Point::default()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use math::{point, vector};
    use ray::Ray;

    fn aab(x0: f64, y0: f64, z0: f64, x1: f64, y1: f64, z1: f64)
        -> AxisAlignedBox
    {
        AxisAlignedBox {
            lower: point(x0, y0, z0),
            upper: point(x1, y1, z1)
        }
    }

    #[test]
    fn union() {
        let test_cases = vec![
            (aab(1.0, 2.0, 3.0, 4.0, 5.0, 6.0),
             aab(7.0, 8.0, 9.0, 10.0, 11.0, 12.0),
             aab(1.0, 2.0, 3.0, 10.0, 11.0, 12.0),
             "a < b"
            ),
            (aab(7.0, 8.0, 9.0, 10.0, 11.0, 12.0),
             aab(1.0, 2.0, 3.0, 4.0, 5.0, 6.0),
             aab(1.0, 2.0, 3.0, 10.0, 11.0, 12.0),
             "a > b"
            ),
        ];

        for (a, b, expected, name) in test_cases {
            let actual = a.union(&b);
            assert_eq!(actual, expected, "{}: expected {:?}, got {:?}",
                       name, expected, actual);
        }
    }

    #[test]
    fn intersection() {
        let b = AxisAlignedBox {
            lower: point(-0.5, -0.5, -0.5),
            upper: point( 0.5,  0.5,  0.5)
        };

        let test_cases = vec![
            ("+z", Ray::new(point(0.0, 0.0, -4.0), vector(0.0, 0.0, 1.0)),
             Some(3.5)),
            ("-z", Ray::new(point(0.0, 0.0, 5.0), vector(0.0, 0.0, -1.0)),
             Some(4.5)),
            ("+y", Ray::new(point(0.0, -6.0, 0.0), vector(0.0, 1.0, 0.0)),
             Some(5.5)),
            ("-y", Ray::new(point(0.0, 7.0, 0.0), vector(0.0, -1.0, 0.0)),
             Some(6.5)),
            ("+x", Ray::new(point(-8.0, 0.0, 0.0), vector(1.0, 0.0, 0.0)),
             Some(7.5)),
            ("-x", Ray::new(point(9.0, 0.0, 0.0), vector(-1.0, 0.0, 0.0)),
             Some(8.5)),
        ];

        for (name, r, expected) in test_cases {
            let actual = b.intersects(&r);
            assert_eq!(actual, expected, "{}: expected {:?}, got {:?}",
                       name, expected, actual);
        }
    }
}