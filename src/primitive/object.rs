use std::sync::Arc;

use light::Light;
use material::{Finish, Material};
use math::{Point, Transform, Vector};
use primitive::{AxisAlignedBox, Primitive};
use ray::Ray;
use scene::SceneVisitor;

use super::SurfaceInfo;

#[derive(Debug)]
pub struct Object {
    pub primitive: Arc<Primitive>,
    pub material: Material,
    pub transform: Option<Box<Transform>>,
}

impl Object {
    pub fn from(p: Arc<Primitive>) -> Object {
        Object {
            primitive: p,
            transform: None,
            material: Material::default()
        }
    }

    pub fn as_light<'a>(&'a self) -> Option<&'a Light> {
        self.primitive.as_light()
    }

    pub fn intersects(&self, r: Ray) -> Option<Point> {
        let r_ = match self.transform {
            Some(ref t) => r.transform(&t.inverse),
            None => r
        };

        self.primitive.intersects(r_).map(|n| {
            let object_space_point = r_.extend(n);
            match self.transform {
                Some(ref t) => t.matrix * object_space_point,
                None => object_space_point
            }
        })
    }

    /// Gets information about the surface at this point. Behaviour is
    /// undefined the supplied point does not lie on the surface of the
    /// object.
    pub fn surface_at(&self, pt: Point) -> SurfaceInfo {
        // convert the global point into the the local object space
        let local_pt = match self.transform {
            Some(ref t) => t.inverse * pt,
            None => pt
        };

        // sample the surface
        let (colour, finish) = self.material.sample(local_pt);

        // translate the surface info back into global space
        let object_space_normal = self.primitive.normal(local_pt);
        let world_space_normal = match self.transform {
            Some(ref t) => object_space_normal.transform(&t.matrix),
            None => object_space_normal
        };

        SurfaceInfo {
            normal: world_space_normal,
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

    pub fn accept(&self, visitor: &mut SceneVisitor) {
        self.primitive.accept_children(self, visitor)
    }

    /// Creates a bounding box for the object
    pub fn bounding_box(&self) -> AxisAlignedBox {
        use math::point;

        let inner_bb = self.primitive.bounding_box();
        match self.transform {
            None => inner_bb,
            Some(ref t) => {
                let AxisAlignedBox { lower: l, upper: u } = inner_bb;

                let points = [
                    t.matrix * Point::new(l.x, l.y, l.z),
                    t.matrix * Point::new(u.x, l.y, l.z),
                    t.matrix * Point::new(u.x, u.y, l.z),
                    t.matrix * Point::new(l.x, u.y, l.z),
                    t.matrix * Point::new(l.x, l.y, u.z),
                    t.matrix * Point::new(u.x, l.y, u.z),
                    t.matrix * Point::new(u.x, u.y, u.z),
                    t.matrix * Point::new(l.x, u.y, u.z),
                ];

                let (min, max) = points.iter()
                    .skip(1)
                    .fold((points[0], points[0]),
                          |(mut min, mut max), p|{
                              min.x = f64::min(min.x, p.x);
                              min.y = f64::min(min.y, p.y);
                              min.z = f64::min(min.z, p.z);
                              max.x = f64::max(max.x, p.x);
                              max.y = f64::max(max.y, p.y);
                              max.z = f64::max(max.z, p.z);
                              (min, max)
                          });

                AxisAlignedBox { lower: min, upper: max }
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::Object;
    use math::point;
    use std::f64::consts::SQRT_2;

    #[test]
    fn bounding_box() {
        use math::Transform;
        use primitive::AxisAlignedBox;
        use primitive::_box::Box as _Box;
        use material::Material;
        use std::sync::Arc;
        use units::degrees;

        let obj = Object {
            primitive: Arc::new(_Box::default()),
            material: Material::default(),
            transform: Some(Box::new(
                Transform::for_rotation(
                    degrees(0.0).radians(),
                    degrees(45.0).radians(),
                    degrees(0.0).radians()))
            ),
        };

        let bb = obj.bounding_box();
        let expected = AxisAlignedBox{
            lower: point(-SQRT_2/2.0, -0.5, -SQRT_2/2.0),
            upper: point( SQRT_2/2.0,  0.5,  SQRT_2/2.0)
        };

        assert!(bb.lower.approx_eq(expected.lower));
        assert!(bb.upper.approx_eq(expected.upper));
    }
}