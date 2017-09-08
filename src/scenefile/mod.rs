#[macro_use] mod constructs;
mod camera;
mod colour;

use std::f64;
use std::any::Any;
use std::str::{FromStr, from_utf8};
use std::io::prelude::*;
use std::io::{self, Result};
use std::fs::File;
use std::path::Path;
use std::convert::From;
use std::error::Error;

use colour::Colour;
use math::{Point, point, Vector, vector};
use primitive::{self, Primitive, Sphere, Box as _Box};
use camera::Camera;
use light::{Light, PointLight};
use units::{degrees, Degrees};

use scene::Scene;

use nom::{multispace, digit, alpha, alphanumeric, IResult, Err, ErrorKind};

use self::camera::*;
use self::colour::colour;
use self::constructs::*;

// ////////////////////////////////////////////////////////////////////////////
// Parsing tools
// ////////////////////////////////////////////////////////////////////////////

/**
 * A possibly-empty whitespace string
 */
named!(whitespace0< Vec<char> >, many0!(one_of!(" \t\n")));

/**
 * Whitespace string with at least one char.
 */
named!(whitespace1< Vec<char> >, many1!(one_of!(" \t\n")));

// ////////////////////////////////////////////////////////////////////////////
// Primitives
// ////////////////////////////////////////////////////////////////////////////


fn sphere<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Box<Sphere>> {
    let mut result = Box::new(Sphere::default());

    let rval = {
        named_object!(input, "sphere",
            block!(separated_list!(comma,
                alt!(
                    call!(named_value, "radius", real_number, |r| {result.radius = r;}) |
                    call!(named_value, "centre", vector_literal, |c| {result.centre = c;})
                )
            )
        ))
    };

    match rval {
        IResult::Done(i, _) => {
            debug!("{:?}", result);
            IResult::Done(i, result)
        },
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x)
    }
}

fn _box<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Box<_Box>> {
    let mut result = Box::new(_Box::default());

    let rval = {
        named_object!(input, "box",
            block!(separated_list!(comma,
                alt!(
                    call!(named_value, "upper", vector_literal, |u| { result.upper = u;}) |
                    call!(named_value, "lower", vector_literal, |l| { result.lower = l;})
                )
            ))
        )
    };

    match rval {
        IResult::Done(i, _) => {
            debug!("{:?}", result);
            IResult::Done(i, result)
        },
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x)
    }
}

// ////////////////////////////////////////////////////////////////////////////
// lights
// ////////////////////////////////////////////////////////////////////////////

fn point_light<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Box<PointLight>> {
    let mut result = Box::new(PointLight::default());

    let rval = {
        named_object!(input, "point_light",
            block!(separated_list!(comma,
                alt!(
                    call!(named_value, "colour", |i| {colour(i)}, |c| { result.colour = c;}) |
                    call!(named_value, "location", vector_literal, |p| { result.loc = p; })
                )
            )))
    };

    match rval {
        IResult::Done(i, _) => IResult::Done(i, result),
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x)
    }
}

// ////////////////////////////////////////////////////////////////////////////
// top level scene file
// ////////////////////////////////////////////////////////////////////////////

fn primitive<'a>(input: &'a [u8], state: &SceneState) -> IResult<&'a [u8], Box<Primitive>> {
    alt!(input,
        map!(call!(sphere, state), |s| {s as Box<Primitive>}) |
        map!(call!(_box, state), |b| {b as Box<Primitive>}) )
}

fn light<'a>(input: &'a [u8], state: &SceneState) -> IResult<&'a [u8], Box<Light>> {
    //alt!(input, call!(point_light, state))
    map!(input, call!(point_light, state), |p| {p as Box<Light>})
}

fn scene_file<'a>(input: &'a [u8]) -> IResult<&'a [u8], Scene> {
    let mut state = SceneState::default();
    let mut text = input;

    let (mut text, cam) = camera(input, &state)
        .unwrap_or((input, Camera::default()));
    state.scene.camera = cam;

    loop {
        if text.is_empty() {
            return IResult::Done(text, state.scene)
        }

        let rval = ws!(text, alt!(
            map!(call!(primitive, &state), |p| { state.scene.add_object(p); }) |
            map!(call!(light, &state), |l| { state.scene.add_light(l); })
        ));

        match rval {
            IResult::Done(i, _) => (text = i),
            IResult::Error(e) => return IResult::Error(e),
            IResult::Incomplete(x) => return IResult::Incomplete(x)
        }
    }
}

fn to_io_error<E>(error: E) -> io::Error
    where E: Into<Box<Error + Send + Sync>>
{
    io::Error::new(io::ErrorKind::Other, error)
}

fn scene_template(source: &str) -> Result<Scene> {
    use liquid::{self, Context, LiquidOptions, Renderable};

    debug!("Compiling scene template...");
    let template = liquid::parse(source, LiquidOptions::default())
        .map_err(to_io_error)?;
    let mut ctx = Context::new();

    debug!("Rendering template...");
    let scene_text = template.render(&mut ctx)
        .map_err(to_io_error)?
        .unwrap();

    match scene_file(scene_text.as_bytes()) {
        IResult::Done(_, s) => {
            info!("Scene loaded");
            Ok(s)
        },
        IResult::Error(error_kind) => {
            error!("Scene load failed: {:?}", error_kind);
            Err(io::Error::new(
                io::ErrorKind::Other, error_kind.description()))
        },
        IResult::Incomplete(_) => {
            info!("Scene incomplete!");
            Err(io::Error::new(
                io::ErrorKind::Other, "incomplete?"))
        }
    }
}

pub fn load_scene<P: AsRef<Path>>(filename: P) -> Result<Scene> {
    info!("Loading scene from {:?}...", filename.as_ref());
    let mut f = File::open(filename)?;
    let mut source = String::new();
    f.read_to_string(&mut source)?;
    scene_template(&source)
}

#[cfg(test)]
mod test {
    use super::*;
    use nom;

    // ////////////////////////////////////////////////////////////////////////
    //
    // ////////////////////////////////////////////////////////////////////////

    #[test]
    fn parse_sphere() {
        use colour::Colour;
        use math::point;
        use primitive::Sphere;
        use nom::IResult;

        let state = SceneState::default();

        match sphere(b"sphere { radius: 1.2340, centre: {1, 2, 3} }", &state) {
            IResult::Done(_, s) => {
                assert_eq!(s.radius, 1.234);
                assert_eq!(s.centre, point(1.0, 2.0, 3.0));
            },
            IResult::Error(_) | IResult::Incomplete(_) => assert!(false)
        }
    }

    // ////////////////////////////////////////////////////////////////////////
    //
    // ////////////////////////////////////////////////////////////////////////

    #[test]
    fn parse_point_light() {
        use colour::Colour;
        use math::point;
        use light::PointLight;
        use nom::IResult;

        let mut state = SceneState::default();

        match point_light(b"point_light { colour: {0.3, 0.4, 0.5}, location: {1, 2, 3} }", &state) {
            IResult::Done(_, l) => {
                assert_eq!(l.colour, Colour::new(0.3, 0.4, 0.5));
                assert_eq!(l.loc, point(1.0, 2.0, 3.0));
            },
            IResult::Error(_) | IResult::Incomplete(_) => assert!(false)
        }
    }

    #[test]
    fn scene_template() {
        super::load_scene("Hello");
    }

    #[test]
    fn scene_file() { super::load_scene("scenes/example.rg"); }
}