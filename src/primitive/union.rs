use std::sync::Arc;
use log::debug;

use crate::{
    math::{Matrix, Point, Transform, Vector},
    scene::SceneVisitor,
    ray::Ray
};

use super::{AxisAlignedBox, Object, Primitive};

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

    fn accept_children(&self, obj: &Object, v: &mut dyn SceneVisitor) {
        debug!("Union: accept_children!");

        let transform = match obj.transform {
            Some(ref t) => t.as_ref().clone(),
            None => Transform::identity()
        };

        v.push_transform(&transform);

        for child in self.children.iter() {
            v.visit(Arc::clone(child));
        }

        v.pop_transform();
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
    use crate::{
        primitive::Primitive,
        math::{point, vector, IDENTITY},
        ray::Ray
    };

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