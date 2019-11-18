
use crate::{
    math::{self, point, Point, Vector},
    ray::Ray,
};

use super::{AxisAlignedBox, Object, Primitive};

///
/// An axis-aligned box
///
#[derive(Debug)]
pub struct Box(AxisAlignedBox);

impl Box {
    pub fn new(lower: Point, upper: Point) -> Box {
        Box(AxisAlignedBox{ lower, upper })
    }

    pub fn from(b: AxisAlignedBox) -> Box {
        Box(b)
    }

    pub fn lower(&self) -> &Point {
        &self.0.lower
    }

    pub fn upper(&self) -> &Point {
        &self.0.upper
    }
}

impl Default for Box {
    fn default() -> Box {
        Box(AxisAlignedBox {
            lower: point(-0.5, -0.5, -0.5),
            upper: point(0.5, 0.5, 0.5)
        })
    }
}

impl Primitive for Box {
    fn intersects(&self, r: Ray) -> Option<f64> {
        self.0.intersects(&r)
    }

    fn bounding_box(&self) -> AxisAlignedBox {
        self.0.clone()
    }

    fn normal(&self, pt: Point) -> Vector {
        use math::unit_vectors::*;
        let &Box(ref b) = self;

        const EPSILON: f64 = 1e-10;

        if (pt.x - b.lower.x).abs() < EPSILON { NEG_X }
        else if (pt.x - b.upper.x).abs() < EPSILON { POS_X }
        else if (pt.y - b.lower.y).abs() < EPSILON { NEG_Y }
        else if (pt.y - b.upper.y).abs() < EPSILON { POS_Y }
        else if (pt.z - b.lower.z).abs() < EPSILON { NEG_Z }
        else if (pt.z - b.upper.z).abs() < EPSILON { POS_Z }
        else { panic!("Point not on box: {:?}", pt) }
    }
}