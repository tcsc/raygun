use std::sync::Arc;

use raygun_camera::Camera;
use raygun_material::{Colour, COLOUR_BLACK};
use raygun_math::{Ray, Transform};
use raygun_primitives::{Object, Visitor};

///
/// The toplevel owner of all objects and lights
///
pub struct Scene {
    pub objects: Vec<Arc<Object>>,
    pub camera: Camera,
}

pub struct LightInfo {
    pub transform: Transform,
    pub light: Arc<Object>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            camera: Camera::default(),
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, obj: Object) {
        self.objects.push(Arc::new(obj));
    }

    pub fn visit(&self, v: &mut dyn Visitor) {
        for o in self.objects.iter() {
            v.visit(Arc::clone(o));
            o.accept(v);
        }
    }

    pub fn lights<'a>(&'a self) -> Vec<LightInfo> {
        struct LightVisitor {
            transform_stack: Vec<Transform>,
            lights: Vec<LightInfo>,
        }

        impl Visitor for LightVisitor {
            fn push_transform(&mut self, t: &Transform) {
                let head = self.transform_stack.last().unwrap().clone();
                self.transform_stack.push(head.apply(t));
            }

            fn pop_transform(&mut self) {
                self.transform_stack.pop();
            }

            fn visit(&mut self, obj: Arc<Object>) {
                if obj.as_light().is_some() {
                    let t = self.transform_stack.last().unwrap();
                    self.lights.push(LightInfo {
                        transform: t.clone(),
                        light: obj,
                    });
                }
            }
        }

        let mut visitor = LightVisitor {
            transform_stack: vec![Transform::identity()],
            lights: Vec::new(),
        };

        self.visit(&mut visitor);

        return visitor.lights;
    }

    pub fn sky(&self, _: Ray) -> Colour {
        COLOUR_BLACK
    }
}
