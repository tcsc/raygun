#[macro_use]
mod constructs;
mod camera;
mod colour;
mod lights;
mod material;
mod primitive;

use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::convert::From;
use std::error::Error;

use nom::IResult;
use liquid::{self, Context, LiquidOptions, Renderable};

use camera::Camera;
use scene::Scene;

use self::camera::*;
use self::constructs::*;
use self::primitive::primitive;

// ////////////////////////////////////////////////////////////////////////////
// top level scene file
// ////////////////////////////////////////////////////////////////////////////

fn scene_file<'a>(input: &'a [u8]) -> IResult<&'a [u8], Scene> {
    let mut state = SceneState::default();

    let (text, cam) = camera(input, &state)
        .unwrap_or((input, Camera::default()));

    many1!(text, ws!(call!(primitive, &mut state)))
        .map(|objs| {
            debug!("Parsed {} objects", objs.len());
            Scene { camera: cam, objects: objs }
        })
}

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

                    // uncomment for debug
                    // File::create("scene.rso").unwrap().write(&bytes);

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
