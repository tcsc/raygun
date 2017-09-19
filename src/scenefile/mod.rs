#[macro_use]
mod constructs;
mod camera;
mod colour;
mod lights;
mod material;
mod primitive;

use std::f64;
use std::any::Any;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::convert::From;
use std::error::Error;
use std::str::{FromStr, from_utf8};

use liquid::{self, Context, LiquidOptions, Renderable};

use colour::Colour;
use math::{Point, point, Vector, vector};
use primitive::Object;
use camera::Camera;
use light::{Light, PointLight};
use units::{degrees, Degrees};
use material::Material;

use scene::Scene;

use nom::{multispace, digit, alpha, alphanumeric, IResult, Err, ErrorKind};

use self::camera::*;
use self::colour::colour;
use self::constructs::*;
use self::lights::*;
use self::material::material;
use self::primitive::primitive;

// ////////////////////////////////////////////////////////////////////////////
// top level scene file
// ////////////////////////////////////////////////////////////////////////////

fn scene_file<'a>(input: &'a [u8]) -> IResult<&'a [u8], Scene> {
    let mut state = SceneState::default();
    let mut text = input;

    let (mut text, cam) = camera(input, &state).unwrap_or((input, Camera::default()));

    many1!(text, ws!(call!(primitive, &state)))
        .map(|objs| {
            debug!("Parsed {} objects", objs.len());
            Scene { camera: cam, objects: objs }
        })
}

//fn to_io_error<E>(error: E) -> io::Error
//    where E: Into<Box<Error + Send + Sync>>
//{
//    io::Error::new(io::ErrorKind::Other, error)
//}

pub enum SceneError {
    FileError(io::Error),
    Template(String),
    Scene(Vec<String>)
}

fn to_template_error(e: liquid::Error) -> SceneError {
    SceneError::Template(e.description().to_owned())
}

fn scene_template(source: &str) -> Result<Scene, SceneError> {
    debug!("Compiling scene template...");
    liquid::parse(source, LiquidOptions::default())
        .map_err(to_template_error)
        .and_then(|template| {
            let mut ctx = Context::new();

            debug!("Rendering template...");
            template.render(&mut ctx)
                .map_err(to_template_error)
                .and_then(|maybe_scene_text| {
                    let scene_text = maybe_scene_text.unwrap();
                    let bytes = scene_text.as_bytes().to_vec();

                    File::create("scene.rso").unwrap().write(&bytes);

                    debug!("Parsing scene...");
                    match scene_file(&bytes) {
                        IResult::Done(_, s) =>  Ok(s),
                        IResult::Error(err) => {
                            let mut errors = vec![String::from(err.description())];
                            let mut cause = err.cause();
                            loop {
                                match cause {
                                    Some(err) => {
                                        errors.push(format!("{}", err));
                                        cause = err.cause();
                                    },
                                    None => break Err(SceneError::Scene(errors))
                                }
                            }
                        },
                        IResult::Incomplete(_) => {
                            Err(SceneError::Scene(vec![]))
                        }
                    }
                })
        })
}

pub fn load_scene<P: AsRef<Path>>(filename: P) -> Result<Scene, SceneError> {
    info!("Loading scene from {:?}...", filename.as_ref());

    File::open(filename)
        .map_err(|e| { SceneError::FileError(e) })
        .and_then(|mut f| {
            let mut source = String::new();
            f.read_to_string(&mut source)
                .map_err(|e| { SceneError::FileError(e) })
                .and_then(|_| {
                    scene_template(&source)
                })
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use nom;

    #[test]
    fn scene_template() {
        super::load_scene("Hello");
    }

    #[test]
    fn scene_file() {
        super::load_scene("scenes/example.rg");
    }
}
