#[macro_use] mod constructs;
mod camera;

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

named!(symbol<String>,
    do_parse!(
        head: map_res!(alpha, from_utf8) >>
        tail: many0!(
                map_res!(
                    alt!(alphanumeric | tag!("_") | tag!("-")),
                    from_utf8)) >>
        (tail.iter().fold(head.to_string(), |mut acc, slice| {
            acc.push_str(slice);
            acc
        })
    )
));

fn declaration<'a, T, StoreFn, ParserFn>(
    i: &'a [u8],
    typename: &str,
    parser: ParserFn,
    mut storefn: StoreFn) -> IResult<&'a [u8], ()>
    where
        StoreFn : FnMut(&'a [u8], &str, T) -> IResult<&'a [u8], ()>,
        ParserFn : Fn(&'a [u8]) -> IResult<&'a [u8], T> {
    let parse_result = do_parse!(i,
            ws!(tag!("let")) >> name: ws!(symbol) >>
            ws!(char!('=')) >> value: named_object!(typename, call!(parser)) >>
            (name, value));
    match parse_result {
        IResult::Done(i, (name, value)) => storefn(i, &name, value),
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x),
    }
}


// ////////////////////////////////////////////////////////////////////////////
// Colours
// ////////////////////////////////////////////////////////////////////////////

/**
 * A colour literal of the form {r, g, b}
 */
named!(colour_literal<Colour>, block!(
    do_parse!(rr: real_number >>
              comma >>
              gg: real_number >>
              comma >>
              bb: real_number >>
              (Colour::new(rr, gg, bb)))
));

fn colour_declaration<'a>(input: &'a [u8], scene: &mut SceneState) ->
IResult<&'a [u8], ()> {
    declaration(input, "colour", colour_literal,
                |i, name, value| {
                    let already_exists = scene.colours.contains_key(name);
                    if already_exists {
                        IResult::Error(ErrorKind::Custom(99))
                    } else {
                        debug!("new colour {}", name);
                        scene.colours.insert(String::from(name), value);
                        IResult::Done(i, ())
                    }
                })
}

fn colour_reference<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Colour> {
    map_opt!(input,
        call!(symbol),
        |name| { scene.colours.get(&name).map(|c| *c) })
}

fn colour<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Colour> {
    alt!(input, call!(colour_literal) |
                call!(colour_reference, scene))
}

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

    let lookup_colour = |i| { colour(i, scene) };

    let rval = {
        named_object!(input, "point_light",
            block!(separated_list!(comma,
                alt!(
                    call!(named_value, "colour", |i|{colour(i,scene)}, |c| { result.colour = c;}) |
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
            call!(colour_declaration, &mut state) |
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
    // Basic parsing tools
    // ////////////////////////////////////////////////////////////////////////

    #[test]
    fn parse_digit_string() {
        use nom::IResult;

        assert!(!digit_string(b"").is_done(), "Empty string");
        assert!(!digit_string(b"abcd").is_done(), "Text string");
        assert_eq!(digit_string(b"1234"), IResult::Done(&b""[..], "1234"));
        assert_eq!(digit_string(b"1234a567"), IResult::Done(&b"a567"[..], "1234"));
    }

    #[test]
    fn parse_symbol() {
        use nom::IResult;

        assert!(!symbol(b"").is_done(), "Empty string");
        assert!(!symbol(b"0bcd").is_done(), "Leading digit");

        assert_eq!(symbol(b"some_symbol"), IResult::Done(&b""[..],
                                                         String::from("some_symbol")));

        assert_eq!(symbol(b"symbol_with_d1g8s"),
                   IResult::Done(&b""[..], String::from("symbol_with_d1g8s")));

        assert_eq!(symbol(b"symbol with trailing text"),
                   IResult::Done(&b" with trailing text"[..], String::from("symbol")));
    }

    #[test]
    fn parse_float() {
        use nom::IResult;

        assert!(real_number(b"").is_incomplete(), "Empty string");
        assert_eq!(real_number(b"163"), IResult::Done(&b""[..], 163.0));
        assert_eq!(real_number(b"+163"), IResult::Done(&b""[..], 163.0));
        assert_eq!(real_number(b"-163"), IResult::Done(&b""[..], -163.0));
        assert_eq!(real_number(b"-163"), IResult::Done(&b""[..], -163.0));
        assert_eq!(real_number(b"27.01"), IResult::Done(&b""[..], 27.01));
        assert_eq!(real_number(b"-27.01"), IResult::Done(&b""[..], -27.01));

        assert_eq!(real_number(b"-12.34 plus some other text"),
                   IResult::Done(&b" plus some other text"[..], -12.34));

        assert_eq!(real_number(b"42 plus some other text"),
                   IResult::Done(&b" plus some other text"[..], 42.0));
    }

    #[test]
    fn parse_vector_literal() {
        use math::vector;
        use nom::IResult;

        let v = vector(1.0, 0.5, 0.0);
        let expected = IResult::Done(&b""[..], v);

        assert_eq!(vector_literal(b"{1, 0.5, 0}"), expected);
        assert_eq!(vector_literal(b"{ 1.0 , 0.5, 0.0}"), expected);
        assert_eq!(vector_literal(b"{1.0,0.5,0.0 }"), expected);
    }

    #[test]
    fn parse_named_value() {
        use super::{named_value, real_number};
        use nom::IResult;

        let mut f  = 0.0;
        assert_eq!(
            named_value(b"float: 42", "float", real_number, |c| { f = c; }),
            IResult::Done(&b""[..], ()));

        assert_eq!(f, 42.0);
    }


    // ////////////////////////////////////////////////////////////////////////
    // Colour tests
    // ////////////////////////////////////////////////////////////////////////

    #[test]
    fn parse_colour_literal() {
        use colour::Colour;
        use nom::IResult;

        let c = Colour {r: 1.0, g: 0.5, b: 0.0};
        let expected = IResult::Done(&b""[..], c);

        assert_eq!(colour_literal(b"{1, 0.5, 0}"), expected);
        assert_eq!(colour_literal(b"{ 1.0 , 0.5, 0.0}"), expected);
        assert_eq!(colour_literal(b"{1.0,0.5,0.0 }"), expected);
    }

    #[test]
    fn parse_colour_declaration() {
        use colour::Colour;
        use nom::IResult;

        let orange = Colour::new(1.0, 0.5, 0.0);
        let mut state = SceneState::default();
        assert_eq!(colour_declaration(b"let orange = colour {1, 0.5, 0}", &mut state),
                   IResult::Done(&b""[..], ()));
        assert!(state.colours.get("orange").unwrap().approx_eq(orange));

        // redefining orange is not allowed
        assert!(colour_declaration(b"let orange = colour {0, 0, 0}",
                                   &mut state).is_err());

        // something other than a colour is not allowed
        assert!(colour_declaration(b"let orange = material {1, 2, 3}",
                                   &mut state).is_err());

        // a malformed colour is not allowed
        assert!(colour_declaration(b"let orange = colour {}",
                                   &mut state).is_err());
    }

    #[test]
    fn parse_colour_reference() {
        use colour::Colour;
        use nom::IResult;

        let orange = Colour::new(1.0, 0.5, 0.0);
        let mut state = SceneState::default();
        state.colours.insert(String::from("orange"), orange);

        assert_eq!(colour_reference(b"orange", &state), IResult::Done(&b""[..], orange));
        assert!(colour_reference(b"puce", &state).is_err());
        assert!(colour_reference(b"", &state).is_incomplete());
    }

    #[test]
    fn parse_colour() {
        use colour::Colour;
        use nom::IResult;

        let orange = Colour::new(1.0, 0.5, 0.0);
        let red = Colour::new(1.0, 0.0, 0.0);

        let mut state = SceneState::default();
        state.colours.insert(String::from("orange"), orange);

        assert_eq!(colour(b"orange", &state), IResult::Done(&b""[..], orange));
        assert_eq!(colour(b"{1, 0, 0}", &state), IResult::Done(&b""[..], red));
    }


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

        let fucsia = Colour::new(1.0, 0.0, 1.0);
        let mut state = SceneState::default();
        state.colours.insert(String::from("fucsia"), fucsia);

        match point_light(b"point_light { colour: fucsia, location: {1, 2, 3} }", &state) {
            IResult::Done(_, l) => {
                assert_eq!(l.colour, fucsia);
                assert_eq!(l.loc, point(1.0, 2.0, 3.0));
            },
            IResult::Error(_) | IResult::Incomplete(_) => assert!(false)
        }

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