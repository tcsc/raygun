#[macro_use]
mod constructs;
mod camera;
mod colour;
mod lights;
mod material;
mod primitive;
mod transform;

use std::{
    fs::File,
    sync::Arc,
    cell::RefCell,
    io::{self, Read},
    prelude::*,
    path::Path,
    convert::From,
    error::Error,
};

use nom::{
    IResult,
};
use liquid;
use log::{debug, info};

use crate::{
    camera::Camera,
    scene::Scene
};

use self::{
    camera::*,
    constructs::*,
    primitive::*
};

// ////////////////////////////////////////////////////////////////////////////
// top level scene file
// ////////////////////////////////////////////////////////////////////////////

fn scene_file<'a>(input: &'a [u8]) -> IResult<&'a [u8], Scene> {
    use nom::{
        branch::alt
    };

    let state = SceneRef::new(SceneState::default());

    let (text, cam) = camera(state.clone())(input)?;
    primitives(state.clone())(text).map(|(i, objs)| {
        let scene = Scene {
            camera: cam,
            objects: objs
        };
        (i, scene)
    })
}

pub enum SceneError {
    FileError(io::Error),
    Template(String),
    Scene(Vec<String>)
}

fn to_template_error(e: liquid::Error) -> SceneError {
    SceneError::Template(e.to_string())
}

fn scene_template(source: &str) -> Result<Scene, SceneError> {
    debug!("Compiling scene template...");
    liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(source)
        .map_err(to_template_error)
        .and_then(|template| {
            let mut globals = liquid::object!({});

            debug!("Rendering scene template...");
            template.render(&mut globals)
                .map_err(to_template_error)
                .and_then(|scene_text| {
                    let bytes : Vec<u8> = scene_text.as_bytes().to_vec();

                    // uncomment for debug
                    // File::create("scene.rso").unwrap().write(&bytes);

                    debug!("Parsing scene...");
                    match scene_file(&bytes) {
                        IResult::Ok((_, s)) => Ok(s),
                        IResult::Err(nom::Err::Incomplete(_)) => {
                            Err(SceneError::Scene(vec![]))
                        },
                        IResult::Err(nom::Err::Failure((_, err))) => {
                            let errors = vec![String::from(err.description())];
                            Err(SceneError::Scene(errors))
                        },
                        IResult::Err(e) => {
                            Err(SceneError::Scene(vec!["Unknown".to_owned()]))
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
