use primitive::Object;
use camera::Camera;
use colour;
use light::Light;
use colour::Colour;
use ray::Ray;

///
/// The toplevel owner of all objects and lights
///
pub struct Scene {
    pub objects: Vec<Object>,
    pub camera: Camera
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            camera: Camera::default(),
            objects: Vec::new()
        }
    }

    pub fn add_object(&mut self, obj: Object) {
        self.objects.push(obj);
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

    pub fn sky(&self, r: Ray) -> Colour {
        colour::BLACK
    }
}