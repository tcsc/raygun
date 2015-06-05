use primitive::Primitive;
use camera::Camera;

/// The toplevel owner of all objects and lights
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

	pub fn add(&mut self, obj: Box<Primitive>) {
		self.objects.push(obj)
	}
}