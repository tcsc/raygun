use std::boxed::Box;
use std::fmt::Debug;
use downcast;

use ray::Ray;
use math::{Point, Vector};
use light::Light;

///
/// The trait that defines a primitive object
///
pub trait Primitive : downcast::Any + Debug + Sync {
    fn intersects(&self, r: Ray) -> Option<f64>;
    fn normal(&self, pt: Point) -> Vector;

    /// Is this primitive a light?
    fn as_light(&self) -> Option<&Light> {
        None
    }
}

downcast!(Primitive);
