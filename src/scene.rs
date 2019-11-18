use std::sync::Arc;

use crate::{
    primitive::Object,
    camera::Camera,
    colour::{self, Colour},
    light::{Light, PointLight},
    math::Transform,
    ray::Ray
};
 
///
/// The toplevel owner of all objects and lights
///
pub struct Scene {
    pub objects: Vec<Arc<Object>>,
    pub camera: Camera
}

pub trait SceneVisitor {
    fn push_transform(&mut self, t: &Transform) {}
    fn pop_transform(&mut self) {}
    fn visit(&mut self, obj: Arc<Object>);
}

pub struct LightInfo {
    pub transform: Transform,
    pub light: Arc<Object>
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            camera: Camera::default(),
            objects: Vec::new()
        }
    }

    pub fn add_object(&mut self, obj: Object) {
        self.objects.push(Arc::new(obj));
    }

    pub fn visit(&self, v: &mut dyn SceneVisitor) {
        for o in self.objects.iter() {
            v.visit(Arc::clone(o));
            o.accept(v);
        }
    }

    pub fn lights<'a>(&'a self) -> Vec<LightInfo> {
        struct LightVisitor {
            transform_stack: Vec<Transform>,
            lights: Vec<LightInfo>
        }

        impl SceneVisitor for LightVisitor {
            fn push_transform(&mut self, t: &Transform) {
                let head = self.transform_stack.last().unwrap().clone();
                self.transform_stack.push(head.apply(t));
            }

            fn pop_transform(&mut self) {
                self.transform_stack.pop();
            }

            fn visit(&mut self, obj: Arc<Object>) {
                use std::ops::Deref;

                if obj.as_light().is_some() {
                    let t = self.transform_stack.last().unwrap();
                    self.lights.push(LightInfo {
                        transform: t.clone(),
                        light: obj
                    });
                }
            }
        }

        let mut visitor = LightVisitor {
            transform_stack: vec![Transform::identity()],
            lights: Vec::new()
        };

        self.visit(&mut visitor);

        return visitor.lights;
    }

    pub fn sky(&self, r: Ray) -> Colour {
        colour::BLACK
    }
}