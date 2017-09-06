use std::collections::HashMap;
use std::str::from_utf8;

use nom::{multispace, digit, alpha, alphanumeric, IResult, Err, ErrorKind};

use colour::Colour;
use scene::Scene;
use math::{Vector};

// ////////////////////////////////////////////////////////////////////////////
// State data
// ////////////////////////////////////////////////////////////////////////////

/**
 * Hold the state of the scene as it is being parsed
 */
pub struct SceneState {
    pub colours: HashMap<String, Colour>,
    pub scene: Scene,
    pub width: isize,
    pub height: isize
}

impl SceneState {
    pub fn new(width: isize, height: isize) -> SceneState {
        SceneState {
            colours: HashMap::new(),
            scene: Scene::new(),
            width: width,
            height: height
        }
    }
}

impl Default for SceneState {
    fn default() -> SceneState {
        SceneState {
            colours: HashMap::new(),
            scene: Scene::new(),
            width: 1024,
            height: 768
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

pub fn named_value<'a, T, ParserFn, StoreFn>(input: &'a [u8],
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
named!(pub real_number <f64>, do_parse!(
        sign: opt!(alt!(char!('-') | char!('+')))                     >>
        integral: digit_string                                        >>
        fraction: opt!(complete!(preceded!(tag!("."), digit_string))) >>
        ( to_real(sign, integral, fraction) )
    )
);


