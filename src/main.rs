extern crate image;

use std::fs::File;
use std::path::Path;

mod colour;
mod light;
mod math;
mod units;
mod primitive;
mod camera;
mod material;
mod ray;
mod scene;
mod render;

type Pixel = image::Rgba<u8>;

use scene::*;

#[cfg(not(test))]
fn main() {
	let s = make_scene();

	println!("Rendering...");
	let options = render::RenderOptions{ width: 1024, height: 768 };

	if let Some(img) = render::render(&s, options) {
		println!("Saving...");
		match img.save(Path::new("out.png")) {
			Ok(_) => {},
			Err(_) => {}
		}
	}
}

fn make_scene() -> Scene {
	use primitive::Sphere;
	use math::point;

	let mut sc = Scene::new();
	let objs : Vec<Box<primitive::Primitive>> =
		(0..20).map(|n| n as f64)
			   .map(|n| Sphere::new(point(n - 10.0, 0.0, (n - 10.0)*2.0), 1.0))
			   .map(|s| s as Box<primitive::Primitive>)
			   .collect();

	sc.add_objects(objs);
	sc.add_light(math::point(100.0, 100.0, -100.0), colour::white);
	sc.camera = sc.camera.with_loc(0.0, 0.0, -20.0);
	sc
}
