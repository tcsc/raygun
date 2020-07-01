pub mod _box;
pub mod aabb;
pub mod light;
pub mod object;
pub mod plane;
pub mod point_light;
pub mod primitive;
pub mod sphere;
pub mod union;

pub use self::{
    _box::Box,
    aabb::AxisAlignedBox,
    light::Light,
    object::{Object, ObjectList},
    plane::Plane,
    point_light::PointLight,
    primitive::Primitive,
    sphere::Sphere,
    union::Union,
};

use raygun_material::{Colour, Finish};
use raygun_math::{Transform, Vector};
use std::sync::Arc;

///
/// The details of an object's surface at a given point.
///
#[derive(Debug)]
pub struct SurfaceInfo<'a> {
    pub normal: Vector,
    pub colour: Colour,
    pub finish: &'a Finish,
}

pub trait Visitor {
    fn push_transform(&mut self, _t: &Transform) {}
    fn pop_transform(&mut self) {}
    fn visit(&mut self, obj: Arc<Object>);
}
