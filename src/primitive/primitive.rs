use std::boxed::Box;
use std::fmt::Debug;
use downcast;

use ray::Ray;
use math::{Point, Vector};
use light::Light;
use primitive::aabb::AxisAlignedBox;
use primitive::Object;
use scene::SceneVisitor;


///
/// The trait that defines a primitive object
///
pub trait Primitive : downcast::Any + Debug + Send + Sync {
    fn intersects(&self, r: Ray) -> Option<f64>;
    fn normal(&self, pt: Point) -> Vector;

    /// Is this primitive a light?
    fn as_light(&self) -> Option<&Light> {
        None
    }

    /// Bounding box
    fn bounding_box(&self) -> AxisAlignedBox;

    /// Visitor entry point if the object has any children
    fn accept_children(&self, object: &Object, visitor: &mut SceneVisitor) {}
}

downcast!(Primitive);
