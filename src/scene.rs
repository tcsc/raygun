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
    pub lights: Vec<Box<Light>>,
    pub camera: Camera
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            camera: Camera::default(),
            lights: Vec::new(),
            objects: Vec::new()
        }
    }

    pub fn add_objects(&mut self, objects: Vec<Box<Primitive>>) {
        self.objects.extend(objects)
    }

    pub fn add_object(&mut self, obj: Box<Primitive>) {
        self.objects.push(obj)
    }

    pub fn add_light(&mut self, l: Box<Light>) {
        self.lights.push(l)
    }

    pub fn add_point_light(&mut self, pt: Point, colour: Colour) {
        self.lights.push(Box::new(PointLight::new(pt, colour)))
    }
}