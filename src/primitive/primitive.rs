use std::boxed::Box;
use std::fmt::Debug;
use downcast;

use crate::{
    ray::Ray,
    math::{Point, Vector},
    light::Light,
    primitive::{
        aabb::AxisAlignedBox,
        Object
    },
    scene::SceneVisitor
};


///
/// The trait that defines a primitive object
///
pub trait Primitive : downcast::Any + Debug + Send + Sync {
    fn intersects(&self, r: Ray) -> Option<f64>;
    fn normal(&self, pt: Point) -> Vector;

    /// Is this primitive a light?
    fn as_light(&self) -> Option<&dyn Light> {
        None
    }

    /// Bounding box
    fn bounding_box(&self) -> AxisAlignedBox;

    /// Visitor entry point if the object has any children
    fn accept_children(&self, object: &Object, visitor: &mut dyn SceneVisitor) {}
}

downcast!(dyn Primitive);
