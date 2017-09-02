
use math::{self, point, Point, Vector};
use primitive::Primitive;
use ray::Ray;

///
/// An axis-aligned box
///
#[derive(Debug)]
pub struct Box {
    pub lower: Point,
    pub upper: Point
}

impl Box {
    pub fn new(lower: Point, upper: Point) -> Box {
        Box { lower: lower, upper: upper }
    }
}

impl Default for Box {
    fn default() -> Box {
        Box {
            lower: point(-0.5, -0.5, -0.5),
            upper: point(0.5, 0.5, 0.5)
        }
    }
}

#[inline]
fn sort<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

impl Primitive for Box {
    fn intersects(&self, r: Ray) -> Option<f64> {

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
        }
        else if t_min > t_max {
            // Ray does not intersect box
            None
        }
        else {
            Some(t_min)
        }
    }

    fn normal(&self, pt: Point) -> Vector {
        use math::unit_vectors::*;

        const EPSILON: f64 = 1e-10;

        if (pt.x - self.lower.x).abs() < EPSILON { NEG_X }
        else if (pt.x - self.upper.x).abs() < EPSILON { POS_X }
        else if (pt.y - self.lower.y).abs() < EPSILON { NEG_Y }
        else if (pt.y - self.upper.y).abs() < EPSILON { POS_Y }
        else if (pt.z - self.lower.y).abs() < EPSILON { NEG_Z }
        else if (pt.z - self.upper.z).abs() < EPSILON { POS_Z }
        else { panic!("Point not on box: {:?}", pt) }
    }
}