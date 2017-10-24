pub use primitive::aabb::AxisAlignedBox;
pub use primitive::primitive::Primitive;
pub use primitive::sphere::Sphere;
pub use primitive::plane::Plane;
pub use primitive::union::Union;
pub use primitive::_box::Box;
pub use primitive::object::Object;

pub mod aabb;
pub mod _box;
pub mod union;
pub mod object;
pub mod primitive;
pub mod plane;
pub mod sphere;

use std::boxed;
use std::sync::Arc;

use colour::Colour;
use material::Finish;
use math::Vector;

///
/// The details of an object's surface at a given point.
///
#[derive(Debug)]
pub struct SurfaceInfo<'a> {
    pub normal: Vector,
    pub colour: Colour,
    pub finish: &'a Finish
}