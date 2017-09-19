use std::collections::HashMap;
use std::str::from_utf8;

use nom::{multispace, digit, alpha, alphanumeric, IResult, Err, ErrorKind};

use colour::Colour;
use material::Material;
use primitive::{Object, Primitive};
use scene::Scene;
use math::{Matrix, Vector};

// ////////////////////////////////////////////////////////////////////////////
// State data
// ////////////////////////////////////////////////////////////////////////////

/**
 * Hold the state of the scene as it is being parsed
 */
pub struct SceneState {
    pub width: isize,
    pub height: isize,
}

impl SceneState {
    pub fn new(width: isize, height: isize) -> SceneState {
        SceneState {
            width: width,
            height: height,
        }
    }
}

impl Default for SceneState {
    fn default() -> SceneState {
        SceneState {
            width: 1024,
            height: 768,
        }
    }
}

/**
 * A comma (potentially) surrounded by whitespace
 */
named!(pub comma<char>, ws!(char!(',')));

/**
 * A brace-delimited block
 */
#[macro_export]
macro_rules! block {
    ($i:expr, $submac:ident!( $($args:tt)* )) => (
        delimited!(
            $i,
            ws!(char!('{')),
            $submac!($($args)*),
            ws!(char!('}'))
        )
    );

    ($i:expr, $f:expr) => (
        delimited!(
            $i,
            ws!(char!('{')),
            call!($f),
            ws!(char!('}'))
        )
    );
}

#[macro_export]
macro_rules! named_object {
    ($i:expr, $name:expr, $submac:ident!( $($args:tt)* )) => (
        ws!($i, preceded!(ws!(tag!($name)), $submac!($($args)*)))
    );

    ($i:expr, $name:expr, $f:expr) => (
        ws!($i, preceded!(ws!(tag!($name))), ws!(call!($f)))
    );
}

#[macro_export]
macro_rules! set {
    ($t:expr) => {
        |__v__| { $t = __v__; }
    }
}

pub fn named_value<'a, T, ParserFn, StoreFn>(input: &'a [u8],
                                             name: &str,
                                             parser: ParserFn,
                                             mut storefn: StoreFn)
                                             -> IResult<&'a [u8], ()>
    where StoreFn: FnMut(T) -> (),
          ParserFn: Fn(&'a [u8]) -> IResult<&'a [u8], T>
{
    do_parse!(input, ws!(tag!(name)) >>
                     ws!(char!(':')) >>
                     value: ws!(call!(parser)) >> (value))
        .map(|value| {
            storefn(value);
            ()
        })
}

/**
 * A vector literal of the form {x, y, z}
 */
named!(pub vector_literal <Vector>, block!(
    do_parse!(
        xx: real_number >>
        comma >>
        yy: real_number >>
        comma >>
        zz: real_number >>
        ( Vector::new(xx, yy, zz) ))
));

// ////////////////////////////////////////////////////////////////////////////
// Parsing numbers
// ////////////////////////////////////////////////////////////////////////////

/**
 * A string of 0-9 digts.
 */
named!(pub digit_string<&str>, map_res!(digit, from_utf8));

fn to_real(sign: Option<char>, integral: &str, fraction: Option<&str>) -> f64 {
    let s = match sign {
        Some('+') | None => 1.0,
        Some('-') => -1.0,
        Some(c) => panic!("Unexpected sign char: {:?}", c),
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
named!(pub real_number <f64>, do_parse!(
        sign: opt!(alt!(char!('-') | char!('+')))                     >>
        integral: digit_string                                        >>
        fraction: opt!(complete!(preceded!(tag!("."), digit_string))) >>
        ( to_real(sign, integral, fraction) )
    )
);

pub fn as_object<PrimitiveT: Primitive>(p: PrimitiveT,
                                        m: Material,
                                        t: Matrix) -> Object {
    Object::new(Box::new(p) as Box<Primitive>, m, t)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_named_value() {
        use super::{named_value, real_number};
        use nom::IResult;

        let mut f = 0.0;
        assert_eq!(named_value(b"float: 42", "float", real_number, |c| { f = c; }),
                   IResult::Done(&b""[..], ()));

        assert_eq!(f, 42.0);
    }

    #[test]
    fn parse_digit_string() {
        use nom::IResult;

        assert!(!digit_string(b"").is_done(), "Empty string");
        assert!(!digit_string(b"abcd").is_done(), "Text string");
        assert_eq!(digit_string(b"1234"), IResult::Done(&b""[..], "1234"));
        assert_eq!(digit_string(b"1234a567"),
                   IResult::Done(&b"a567"[..], "1234"));
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
}
