pub mod aabb;
pub mod _box;
pub mod union;
pub mod object;
pub mod primitive;
pub mod plane;
pub mod sphere;
pub mod light;
pub mod point_light;

pub use self::{
    aabb::AxisAlignedBox,
    primitive::Primitive,
    sphere::Sphere,
    plane::Plane,
    union::Union,
    _box::Box,
    object::{Object, ObjectList},
    light::Light,
    point_light::PointLight
};

use std::sync::Arc;
use raygun_material::{Colour, Finish};
use raygun_math::{Transform, Vector};

///
/// The details of an object's surface at a given point.
///
#[derive(Debug)]
pub struct SurfaceInfo<'a> {
    pub normal: Vector,
    pub colour: Colour,
    pub finish: &'a Finish
}

pub trait Visitor {
    fn push_transform(&mut self, _t: &Transform) {}
    fn pop_transform(&mut self) {}
    fn visit(&mut self, obj: Arc<Object>);
}