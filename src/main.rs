#![allow(unused)]

#[macro_use]
extern crate downcast;

use std::error::Error;
use std::path::PathBuf;

use simplelog::{TermLogger, TerminalMode, LevelFilter, Config};
use log::{self, info, debug, error};

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

use scenefile::SceneError;

#[cfg(not(test))]
fn main() {
    use std::process::exit;

    TermLogger::init(
        log::LevelFilter::Debug,
        Config::default(), 
        TerminalMode::Stdout).unwrap();

    let args = parse_args();
    info!("Dimensions {} x {}", args.width, args.height);

    let s = scenefile::load_scene(args.scene_file)
        .unwrap_or_else(|err| {
            error!("Scene file loading failed:");
            match err {
                SceneError::FileError(e) => {
                    error!("File IO error: {}", e.description());
                },
                SceneError::Template(s) => {
                    error!("Template parse error: {}", s);
                },
                SceneError::Scene(errs) => {
                    for e in errs {
                        error!("{}", e)
                    }
                }
            };
            exit(1);
        });

    let options = render::RenderOptions {
        width: args.width,
        height: args.height
    };

    info!("Starting render...");
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
    use argparse::{ArgumentParser, Store};

    let mut result = Args {
        width: 640,
        height: 480,
        scene_file: PathBuf::default(),
        output_file: PathBuf::default()
    };

    let mut scene_file = String::new();
    let mut image_file = String::from("render.png");

    /* Artificial scope to limit borrows */ {
        let mut parser = ArgumentParser::new();

        parser.refer(&mut result.width)
            .add_option(&["-w", "--width"], Store, "Image width. Defaults to 640.")
            .metavar("INT");

        parser.refer(&mut result.height)
            .add_option(&["-h", "--height"], Store, "Image height. Defaults to 480.")
            .metavar("INT");

        parser.refer(&mut image_file)
            .add_option(&["-o", "--output"], Store, "Output image file")
            .metavar("FILE");

        parser.refer(&mut scene_file)
            .add_argument("FILE", Store, "The scene file")
            .required()
            .metavar("FILE");

        parser.parse_args_or_exit();
    }

    // repack the values that argparse won't pick up for us
    result.scene_file = PathBuf::from(scene_file);
    result.output_file = PathBuf::from(image_file);
    result
}