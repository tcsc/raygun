use downcast::*;
use std::fmt::Debug;

use crate::{AxisAlignedBox, Light, Object};
use raygun_math::{Point, Ray, Vector};

///
/// The trait that defines a primitive object
///
pub trait Primitive: downcast::Any + Debug + Send + Sync {
    fn intersects(&self, r: Ray) -> Option<f64>;
    fn normal(&self, pt: Point) -> Vector;

    /// Is this primitive a light?
    fn as_light(&self) -> Option<&dyn Light> {
        None
    }

    /// Bounding box
    fn bounding_box(&self) -> AxisAlignedBox;

    /// Visitor entry point if the object has any children
    fn accept_children(&self, _object: &Object, _visitor: &mut dyn crate::Visitor) {}
}

downcast!(dyn Primitive);
