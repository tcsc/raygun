extern crate argparse;
extern crate image;
#[macro_use] extern crate log;
#[macro_use] extern crate nom;
extern crate liquid;
extern crate simplelog;

use std::str;
use std::path::{Path, PathBuf};

use simplelog::{TermLogger, LogLevelFilter, Config};

mod colour;
mod light;
mod math;
mod units;
mod primitive;
mod camera;
mod material;
mod ray;
mod scene;
mod scenefile;
mod render;

use scene::*;

#[cfg(not(test))]
fn main() {
    use std::process::exit;

    TermLogger::init(LogLevelFilter::Debug, Config::default()).unwrap();

    let args = parse_args();
    info!("Dimensions {} x {}", args.width, args.height);

    let mut s = scenefile::load_scene(args.scene_file)
        .unwrap_or_else(|e| {
            error!("Scene file loading failed {:?}", e);
            exit(1);
        });

    s.camera = s.camera.with_loc(0.0, 0.0, -20.0);

    let options = render::RenderOptions {
        width: args.width,
        height: args.height
    };

    if let Some(img) = render::render(&s, options) {
        info!("Saving to {:?}...", args.output_file);
        match img.save(args.output_file) {
            Ok(_) => {},
            Err(_) => {}
        }
    }
}

struct Args {
    width: isize,
    height: isize,
    scene_file: PathBuf,
    output_file: PathBuf
}

fn parse_args() -> Args {
    use std::str::FromStr;
    use argparse::{self, ArgumentParser, Store, StoreOption};

    let mut result = Args {
        width: 640,
        height: 480,
        scene_file: PathBuf::default(),
        output_file: PathBuf::from(("render.png"))
    };

    let mut scene_file = String::new();
    let mut image_file : Option<String> = None;

    /* Artificial scope to limit borrows */ {
        let mut parser = ArgumentParser::new();

        parser.refer(&mut result.width)
            .add_option(&["-w", "--width"], Store, "Image width. Defaults to 640.")
            .metavar("INT");

        parser.refer(&mut result.height)
            .add_option(&["-h", "--height"], Store, "Image height. Defaults to 480.")
            .metavar("INT");

        parser.refer(&mut image_file)
            .add_option(&["-o", "--output"], StoreOption, "Output image file")
            .metavar("FILE");

        parser.refer(&mut scene_file)
            .add_argument("FILE", Store, "The scene file")
            .required()
            .metavar("FILE");

        parser.parse_args_or_exit();
    }

    // repack the values that argparse won't pick up for us
    result.scene_file = PathBuf::from(scene_file);
    result.scene_file = image_file.map(PathBuf::from)
                                  .unwrap_or(result.scene_file);
    result
}

fn make_scene() -> Scene {
	use primitive::Sphere;
	use math::point;

	let mut sc = Scene::new();
	let objs : Vec<Box<primitive::Primitive>> =
		(0..20).map(|n| n as f64)
			   .map(|n| Sphere::new(point((n - 10.0)*1.25, 0.0, (n - 10.0)*4.0), 1.0))
			   .map(|s| s as Box<primitive::Primitive>)
			   .collect();

	sc.add_objects(objs);
	sc.add_point_light(math::point(100.0, 000.0, -000.0), colour::WHITE);
	sc.add_point_light(math::point(100.0,   0.0, -100.0), colour::Colour{r: 0.5, g: 0.0, b: 0.0});
	sc.add_point_light(math::point(000.0,   0.0, -100.0), colour::Colour{r: 0.0, g: 0.25, b: 0.0});
	sc.add_point_light(math::point(-100.0,  0.0, -100.0), colour::Colour{r: 0.0, g: 0.0, b: 0.25});

	sc.camera = sc.camera.with_loc(0.0, 0.0, -20.0);
	sc
}
