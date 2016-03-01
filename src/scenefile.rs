use std::f64;
use std::any::Any;
use std::collections::HashMap;
use std::str::{FromStr, from_utf8};

// use pc::{Parser, ParserExt, parser, space, spaces, digit};
// use pc::primitives::{Consumed, ParseResult, State, ParseError, Stream};
// use pc::combinator::{Between, Token, FnParser, skip_many, many1, token, optional};
// use pc::char::{char};

use colour::Colour;
use math::{Vector, vector};
use primitive::{Primitive, Sphere};
use light::PointLight;

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

macro_rules! named_object {
    ($i:expr, $name:expr, $submac:ident!( $($args:tt)* )) => (
        preceded!($i, preceded!(tag!($name), whitespace0), $submac!($($args)*))
    );

    ($i:expr, $name:expr, $f:expr) => (
        preceded!($i, preceded!(tag!($name), whitespace0), call!($f))
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
    chain!(
        head: map_res!(alpha, from_utf8) ~
        tail: many0!(
                map_res!(
                    alt!(alphanumeric | tag!("_") | tag!("-")),
                    from_utf8)),
        || {
            tail.iter().fold(head.to_string(), |mut acc, slice| {
                acc.push_str(slice);
                acc
            })
        }
    ));

// ////////////////////////////////////////////////////////////////////////////
// Parsing numbers
// ////////////////////////////////////////////////////////////////////////////

/**
 * A string of 0-9 digts.
 */
named!(digit_string<&str>, map_res!(digit, from_utf8));

/**
 * Parses a real number represented as a decimal fraction (as opposed to one in
 * exponential notation)
 */
named!(real_number<f64>,
    chain!(
        sign: alt!(char!('-') | char!('+'))? ~
        integral: digit_string ~
        fraction: complete!(preceded!(tag!("."), digit_string))?,
        || {
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
    )
);

/**
 * A vector literal of the form {x, y, z}
 */
named!(vector_literal<Vector>, block!(
    chain!(
        xx: real_number ~
        whitespace0 ~ char!(',') ~ whitespace0 ~
        yy: real_number ~
        whitespace0 ~ char!(',') ~ whitespace0 ~
        zz: real_number,
        || { Vector::new(xx, yy, zz) })
    ));

fn declaration<'a, T, StoreFn, ParserFn>(
        i: &'a [u8],
        typename: &str,
        parser: ParserFn,
        mut storefn: StoreFn) -> IResult<&'a [u8], ()>
    where
        StoreFn : FnMut(&'a [u8], &str, T) -> IResult<&'a [u8], ()>,
        ParserFn : Fn(&'a [u8]) -> IResult<&'a [u8], T> {
    let result = chain!(i,
                        tag!("let") ~ whitespace1 ~
                        name: symbol ~ whitespace1 ~
                        tag!("=") ~ whitespace1 ~
                        value: named_object!(typename, call!(parser)),
                        || { (name, value) });
    match result {
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
    chain!(
        rr: real_number ~
        whitespace0 ~ char!(',') ~ whitespace0 ~
        gg: real_number ~
        whitespace0 ~ char!(',') ~ whitespace0 ~
        bb: real_number,
        || { Colour::new(rr, gg, bb) })
));

fn colour_declaration<'a>(input: &'a [u8], scene: &mut SceneState) -> IResult<&'a [u8], ()> {
    declaration(input, "colour", colour_literal,
                |i, name, value| {
                    let already_exists = scene.colours.contains_key(name);
                    if already_exists {
                        IResult::Error(Err::Code(ErrorKind::Custom(99)))
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
    alt!(input, call!(colour_literal) | call!(colour_reference, scene))
}

// ////////////////////////////////////////////////////////////////////////////
// Primitives
// ////////////////////////////////////////////////////////////////////////////

fn sphere<'a>(input: &'a [u8]) -> IResult<&'a [u8], f64> {
    named_object!(input, "sphere", block!(real_number))
}

// pub fn parse_scenefile<'a>(input: &'a [u8]) -> IResult<&'a [u8], Scene>  {
//     let mut s = Scene::new();
//     chain!(
//     //     many0!(
//     //         alt!()
//     //     )
//     // )

// }

#[cfg(test)]
mod test {
    use nom;

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
        assert!(colour_reference(b"", &state).is_err());
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
        use super::sphere;

        println!("S1: {:?}", sphere(b"sphere {1.2340}"));
    }
}