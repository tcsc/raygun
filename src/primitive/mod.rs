pub use primitive::primitive::Primitive;
pub use primitive::sphere::Sphere;
pub use primitive::group::Group;
pub use primitive::_box::Box;

pub mod primitive;
pub mod sphere;
pub mod group;
pub mod _box;

use std::borrow::Borrow;
use std::boxed;

use colour::Colour;
use light::Light;
use math::{Point, Matrix, Vector};
use material::{Finish, Material};
use ray::Ray;

#[derive(Debug)]
pub struct Object {
    primitive: boxed::Box<Primitive>,
    transform: Matrix,
    material: Material
}

///
/// The details of an object's surface at a given point.
///
#[derive(Debug)]
pub struct SurfaceInfo<'a> {
    pub normal: Vector,
    pub colour: Colour,
    pub finish: &'a Finish
}

impl Object {
    pub fn new(p: boxed::Box<Primitive>,
               m: Material,
               t: Matrix ) -> Object {
        Object {
            primitive: p,
            transform: t,
            material: m
        }
    }

    pub fn from(p: boxed::Box<Primitive>) -> Object {
        Object {
            primitive: p,
            transform: Matrix::default(),
            material: Material::default()
        }
    }

    pub fn as_light<'a>(&'a self) -> Option<&'a Light> {
        self.primitive.as_light()
    }

    pub fn intersects(&self, r: Ray) -> Option<f64> {
        self.primitive.intersects(r)
    }

    /// Gets information about the surface at this point. Behaviour is
    /// undefined the supplied point does not lie on the surface of the
    /// object.
    pub fn surface_at(&self, pt: Point) -> SurfaceInfo {
        let (colour, finish) = self.material.sample(pt);
        SurfaceInfo {
            normal: self.primitive.normal(pt),
            colour: colour,
            finish: finish
        }
    }

    /// Fetch a reference to the underlying concrete primitive, assuming
    /// you know what type it is in advance, that is...
    /// Mainly useful for testing.
    pub fn as_primitive<P: Primitive>(&self) -> Option<&P> {
        self.primitive.downcast_ref::<P>().ok()
    }
}

