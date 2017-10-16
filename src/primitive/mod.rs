pub use primitive::aabb::AxisAlignedBox;
pub use primitive::primitive::Primitive;
pub use primitive::sphere::Sphere;
pub use primitive::plane::Plane;
pub use primitive::group::Group;
pub use primitive::_box::Box;

pub mod aabb;
pub mod _box;
pub mod group;
pub mod primitive;
pub mod plane;
pub mod sphere;

use std::boxed;
use std::sync::Arc;

use colour::Colour;
use light::Light;
use math::{self, Point, Matrix, Vector, Transform};
use material::{Finish, Material};
use ray::Ray;

#[derive(Debug)]
pub struct Object {
    primitive: boxed::Box<Primitive>,
    material: Material,
    transform: Arc<Transform>,
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
               t: Arc<Transform> ) -> Object {
        Object {
            primitive: p,
            transform: t,
            material: m
        }
    }

    pub fn from(p: boxed::Box<Primitive>) -> Object {
        Object {
            primitive: p,
            transform: Arc::new(Transform::default()),
            material: Material::default()
        }
    }

    pub fn as_light<'a>(&'a self) -> Option<&'a Light> {
        self.primitive.as_light()
    }

    pub fn intersects(&self, r: Ray) -> Option<Point> {
        let r_ = r.transform(&self.transform.inverse);
        self.primitive.intersects(r_).map(|n| {
            self.transform.matrix * r_.extend(n)
        })
    }

    /// Gets information about the surface at this point. Behaviour is
    /// undefined the supplied point does not lie on the surface of the
    /// object.
    pub fn surface_at(&self, pt: Point) -> SurfaceInfo {
        // convert the global point into the the local object space
        let local_pt = self.transform.inverse * pt;

        // sample the surface
        let (colour, finish) = self.material.sample(local_pt);

        // translate the surface info back into global space
        SurfaceInfo {
            normal: {
                let n = self.primitive.normal(local_pt);
                n.transform(&self.transform.matrix).normalize()
            },
            colour,
            finish
        }
    }

    /// Fetch a reference to the underlying concrete primitive, assuming
    /// you know what type it is in advance, that is...
    /// Mainly useful for testing.
    pub fn as_primitive<P: Primitive>(&self) -> Option<&P> {
        self.primitive.downcast_ref::<P>().ok()
    }

    pub fn transform<'a>(&'a self) -> &'a Transform {
        use std::ops::Deref;
        self.transform.deref()
    }

    /// Creates a bounding box for the object
    pub fn bounding_box(&self) -> AxisAlignedBox {
        use math::point;

        let b = self.primitive.bounding_box();
        let tl = self.transform.matrix * b.lower;
        let tu = self.transform.matrix * b.upper;

        let (min_x, max_x) = math::sort(tl.x, tu.x);
        let (min_y, max_y) = math::sort(tl.y, tu.y);
        let (min_z, max_z) = math::sort(tl.z, tu.z);

        AxisAlignedBox {
            lower: point(min_x, min_y, min_z),
            upper: point(max_x, max_y, max_z)
        }
    }
}

#[cfg(test)]
mod test {
    use super::Object;
    use math::point;

    #[test]
    fn bounding_box() {
        use math::Transform;
        use primitive::AxisAlignedBox;
        use primitive::_box::Box as _Box;
        use material::Material;
        use std::sync::Arc;

        let obj = Object {
            primitive: Box::new(_Box::default()),
            material: Material::default(),
            transform: Arc::new(
                Transform::default().scale(2.0, 1.0, 1.0)
            ),
        };

        let bb = obj.bounding_box();
        let expected = AxisAlignedBox{
            lower: point(-1.0, -0.5, -0.5),
            upper: point(1.0, 0.5, 0.5)
        };
        assert_eq!(bb, expected);
    }
}