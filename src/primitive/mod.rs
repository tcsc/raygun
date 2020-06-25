pub use self::{
    aabb::AxisAlignedBox,
    primitive::Primitive,
    sphere::Sphere,
    plane::Plane,
    union::Union,
    _box::Box,
    object::{Object, ObjectList}
};

use log::debug;

pub mod aabb;
pub mod _box;
pub mod union;
pub mod object;
pub mod primitive;
pub mod plane;
pub mod sphere;

use std::{
    boxed,
    sync::Arc
};

use crate::{
    colour::Colour,
    material::Finish,
    math::Vector,
};

///
/// The details of an object's surface at a given point.
///
#[derive(Debug)]
pub struct SurfaceInfo<'a> {
    pub normal: Vector,
    pub colour: Colour,
    pub finish: &'a Finish
}