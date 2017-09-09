use primitive::Primitive;
use camera::Camera;
use light::{Light, PointLight};
use colour::Colour;
use math::{Point};

///
/// The toplevel owner of all objects and lights
///
pub struct Scene {
    pub objects: Vec<Box<Primitive>>,
    pub camera: Camera
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            camera: Camera::default(),
            objects: Vec::new()
        }
    }

    pub fn add_objects(&mut self, objects: Vec<Box<Primitive>>) {
        self.objects.extend(objects)
    }

    pub fn add_object(&mut self, obj: Box<Primitive>) {
        self.objects.push(obj)
    }

    pub fn lights<'a>(&'a self) -> Vec<&'a Light> {
        let mut result = Vec::new();
        for obj in self.objects.iter() {
            if let Some(l) = obj.as_light() {
                result.push(l);
            }
        }
        return result;
    }
}