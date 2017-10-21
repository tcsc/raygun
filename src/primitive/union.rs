use std::sync::Arc;
use math::{Matrix, Point, Vector, IDENTITY};

use primitive::{AxisAlignedBox, Object, Primitive, SceneVisitor};
use ray::Ray;

#[derive(Debug)]
pub struct Union {
    pub children: Vec<Arc<Object>>
}

impl Union {
    pub fn new() -> Union {
        Union::default()
    }
}

impl Primitive for Union {
    fn intersects(&self, r: Ray) -> Option<f64> {
        None
    }

    fn normal(&self, pt: Point) -> Vector {
        Vector::default()
    }

    fn bounding_box(&self) -> AxisAlignedBox {
        let zero = AxisAlignedBox::default();
        self.children
            .iter()
            .fold(zero, |acc, b| acc.union(&b.bounding_box()))
    }

    fn accept(&self, obj: &Object, v: &mut SceneVisitor) {
        v.visit_union(obj, self);
        for child in self.children.iter() {
            child.accept(v);
        }
    }
}

impl Default for Union {
    fn default() -> Union {
        Union {
            children: Vec::new()
        }
    }
}

#[cfg(test)]
mod test {
    use super::Union;
    use primitive::Primitive;
    use math::{point, vector, IDENTITY};
    use ray::Ray;

    #[test]
    fn default() {
        let g = Union::default();
        assert_eq!(g.children.len(), 0);
    }

    #[test]
    fn nothing_intersects() {
        let g = Union::default();
        let r = Ray::new(point(0.0, 0.0, 0.0), vector(0.0, 1.0, 0.0));
        assert_eq!(g.intersects(r), None);
    }
}