use std::any::Any;
use std::boxed::Box;
use ray::Ray;
use math::{Point, Vector};

///
/// The trait that defines a primitive object
///
pub trait Primitive {
    fn intersects(&self, r: Ray) -> Option<f64>;
    fn normal(&self, pt: Point) -> Vector;
}