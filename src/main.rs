extern crate image;

use std::fs::File;
use std::path::Path;

mod math;
mod units;
mod primitive;
mod ray;
mod camera;
mod scene;
mod render;

type Pixel = image::Rgba<u8>;

use scene::*;

#[cfg(not(test))]
fn main() {
	let s = make_scene();

	let options = render::RenderOptions{ width: 1024, height: 768 };

	if let Some(img) = render::render(&s, options) {
		match img.save(Path::new("out.png")) {
			Ok(_) => {},
			Err(_) => {}
		}
	}
}

fn make_scene() -> Scene {
	let mut sc = Scene::new();
	let s = Box::new(primitive::Sphere::default());
	sc.add(s);
	sc
}
