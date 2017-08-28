use std::f64;
use std::any::Any;
use std::collections::HashMap;
use std::str::{FromStr, from_utf8};
use std::io::prelude::*;
use std::io::{self, Result};
use std::fs::File;
use std::path::Path;
use std::convert::From;
use std::error::Error;

use colour::Colour;
use math::{Point, Vector, vector};
use primitive::{Primitive, Sphere};
use light::{Light, PointLight};

use scene::Scene;

use nom::{multispace, digit, alpha, alphanumeric, IResult, Err, ErrorKind};

// ////////////////////////////////////////////////////////////////////////////
// State data
// ////////////////////////////////////////////////////////////////////////////

/**
 * Hold the state of the scene as it is being parsed
 */
struct SceneState {
    colours: HashMap<String, Colour>,
    scene: Scene
}

impl SceneState {
    fn new() -> SceneState {
        SceneState {
            colours: HashMap::new(),
            scene: Scene::new()
        }
    }
}

// ////////////////////////////////////////////////////////////////////////////
// Parsing tools
// ////////////////////////////////////////////////////////////////////////////

//fn block<'a, T>(input: &'a[u8], p: Parser) -> IResult<&[u8], T>
//    where
//        Parser: Fn(&'a[u8]) -> IResult<&'a[u8], T>
//{
//    ws!(delimited!(input, char!('{'), call!(p), char!('}')))
//}

/**
 * A brace-delimited block
 */
macro_rules! block {
    ($i:expr, $submac:ident!( $($args:tt)* )) => (
        delimited!(
            $i,
            preceded!(char!('{'), whitespace0),
            $submac!($($args)*),
            preceded!(whitespace0, char!('}'))
        )
    );

    ($i:expr, $f:expr) => (
        delimited!(
            $i,
            preceded!(char!('{'), whitespace0),
            call!($f),
            preceded!(whitespace0, char!('}'))
        )
    );
}

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

/**
 * A comma (potentially) surrounded by whitespace
 */
named!(comma<char>, ws!(char!(',')));

macro_rules! named_object {
    ($i:expr, $name:expr, $submac:ident!( $($args:tt)* )) => (
        preceded!($i, preceded!(tag!($name), whitespace0), $submac!($($args)*))
    );

    ($i:expr, $name:expr, $f:expr) => (
        preceded!($i, preceded!(tag!($name), whitespace0), call!($f))
    );
}

// ////////////////////////////////////////////////////////////////////////////
// Parsing numbers
// ////////////////////////////////////////////////////////////////////////////

/**
 * A string of 0-9 digts.
 */
named!(digit_string<&str>, map_res!(digit, from_utf8));

fn to_real(sign: Option<char>, integral: &str, fraction: Option<&str>) -> f64 {
    let s = match sign {
        Some('+') | None => 1.0,
        Some('-') => -1.0,
        Some(c) => panic!("Unexpected sign char: {:?}", c)
    };

    let i = integral.parse::<i64>().unwrap() as f64;
    let f = match fraction {
        None => 0.0,
        Some(digits) => {
            let val = digits.parse::<i64>().unwrap() as f64;
            let scale = (10.0 as f64).powi(digits.len() as i32);
            val / scale
        }
    };
    (s * (i + f))
}

/**
 * Parses a real number represented as a decimal fraction (as opposed to one in
 * exponential notation)
 */
named!(real_number <f64>, do_parse!(
        sign: opt!(alt!(char!('-') | char!('+')))                     >>
        integral: digit_string                                        >>
        fraction: opt!(complete!(preceded!(tag!("."), digit_string))) >>
        ( to_real(sign, integral, fraction) )
    )
);

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

fn named_value<'a, T, ParserFn, StoreFn>(input: &'a [u8],
                                         name: &str,
                                         parser: ParserFn,
                                         mut storefn: StoreFn)
        -> IResult<&'a [u8], ()>
    where
        StoreFn : FnMut(T) -> (),
        ParserFn : Fn(&'a [u8]) -> IResult<&'a [u8], T> {
    let result = do_parse!(
            input,
            ws!(tag!(name)) >> ws!(char!(':')) >> value: ws!(call!(parser)) >>
            (value));
    match result {
        IResult::Done(i, value) => {
            storefn(value);
            IResult::Done(i, ())
        },
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x)
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

/**
 * A vector literal of the form {x, y, z}
 */
named!(vector_literal<Vector>, block!(
    do_parse!(
        xx: real_number >>
        comma >>
        yy: real_number >>
        comma >>
        zz: real_number >>
        ( Vector::new(xx, yy, zz) ))
));

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
        IResult::Done(i, _) => IResult::Done(i, result),
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
    //alt!(input, call!(sphere, state))
    map!(input, call!(sphere, state), |s| {s as Box<Primitive>})
}

fn light<'a>(input: &'a [u8], state: &SceneState) -> IResult<&'a [u8], Box<Light>> {
    //alt!(input, call!(point_light, state))
    map!(input, call!(point_light, state), |p| {p as Box<Light>})
}

fn scene_file<'a>(input: &'a [u8]) -> IResult<&'a [u8], Scene> {
    let mut state = SceneState::new();

    let rval = many0!(input, alt!(
        call!(colour_declaration, &mut state) |
        map!(call!(primitive, &state), |p| { state.scene.add_object(p); }) |
        map!(call!(light, &state), |l| { state.scene.add_light(l); })
    ));

    match rval {
        IResult::Done(i, _) => IResult::Done(i, state.scene),
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x)
    }
}

fn to_io_error<E>(error: E) -> io::Error
    where E: Into<Box<Error + Send + Sync>>
{
    io::Error::new(io::ErrorKind::Other, error)
}


fn scene_template(source: &str) -> Result<Scene> {
    use liquid::{self, Context, LiquidOptions, Renderable};

    let template = liquid::parse(source, LiquidOptions::default())
        .map_err(to_io_error)?;
    let mut ctx = Context::new();
    let scene_text = template.render(&mut ctx);

    match scene_file(source.as_bytes()) {
        IResult::Done(_, s) => Ok(s),
        IResult::Error(error_kind) => Err(io::Error::new(
            io::ErrorKind::Other, error_kind.description())),
        IResult::Incomplete(_) => Err(io::Error::new(
            io::ErrorKind::Other, "incomplete?"))
    }
}

fn load_scene<P: AsRef<Path>>(filename: P) -> Result<Scene> {
    let mut f = File::open(filename)?;
    let mut source = String::new();
    f.read_to_string(&mut source)?;
    scene_template(&source)
}

#[cfg(test)]
mod test {
    use nom;

    // ////////////////////////////////////////////////////////////////////////
    // Basic parsing tools
    // ////////////////////////////////////////////////////////////////////////

    #[test]
    fn parse_digit_string() {
        use super::digit_string;
        use nom::IResult;

        assert!(!digit_string(b"").is_done(), "Empty string");
        assert!(!digit_string(b"abcd").is_done(), "Text string");
        assert_eq!(digit_string(b"1234"), IResult::Done(&b""[..], "1234"));
        assert_eq!(digit_string(b"1234a567"), IResult::Done(&b"a567"[..], "1234"));
    }

    #[test]
    fn parse_symbol() {
        use super::symbol;
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
        use super::real_number;

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
        use super::vector_literal;
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
        use super::colour_literal;
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
        use super::{colour_declaration, SceneState};
        use nom::IResult;

        let orange = Colour::new(1.0, 0.5, 0.0);
        let mut state = SceneState::new();
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
        use super::{colour_reference, SceneState};
        use nom::IResult;

        let orange = Colour::new(1.0, 0.5, 0.0);
        let mut state = SceneState::new();
        state.colours.insert(String::from("orange"), orange);

        assert_eq!(colour_reference(b"orange", &state), IResult::Done(&b""[..], orange));
        assert!(colour_reference(b"puce", &state).is_err());
        assert!(colour_reference(b"", &state).is_incomplete());
    }

    #[test]
    fn parse_colour() {
        use colour::Colour;
        use super::{colour, SceneState};
        use nom::IResult;

        let orange = Colour::new(1.0, 0.5, 0.0);
        let red = Colour::new(1.0, 0.0, 0.0);

        let mut state = SceneState::new();
        state.colours.insert(String::from("orange"), orange);

        assert_eq!(colour(b"orange", &state), IResult::Done(&b""[..], orange));
        assert_eq!(colour(b"{1, 0, 0}", &state), IResult::Done(&b""[..], red));
    }


    // ////////////////////////////////////////////////////////////////////////
    //
    // ////////////////////////////////////////////////////////////////////////

    #[test]
    fn parse_sphere() {
        use super::{SceneState, sphere};
        use colour::Colour;
        use math::point;
        use primitive::Sphere;
        use nom::IResult;

        let state = SceneState::new();

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
        use super::{SceneState, point_light};
        use colour::Colour;
        use math::point;
        use light::PointLight;
        use nom::IResult;

        let fucsia = Colour::new(1.0, 0.0, 1.0);
        let mut state = SceneState::new();
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