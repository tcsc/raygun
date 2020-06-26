use std::str::from_utf8;
use std::sync::Arc;
use std::cell::RefCell;

use nom::{
    error::ParseError,
    lib::std::ops::RangeFrom,
    AsChar, 
    Compare,
    InputIter,
    InputLength,
    InputTake,
    InputTakeAtPosition,
    IResult,
    Slice
};  

use raygun_material::Material;
use raygun_primitives::{Object, Primitive};
use raygun_math::{self as math, Matrix, Vector, Transform};

// ////////////////////////////////////////////////////////////////////////////
// State data
// ////////////////////////////////////////////////////////////////////////////

/**
 * Hold the state of the scene as it is being parsed
 */
pub struct SceneState {
    pub width: isize,
    pub height: isize
}

impl SceneState {
    pub fn new(width: isize, height: isize) -> SceneState {
        let base = Arc::new(Transform::default());
        SceneState {
            width: width,
            height: height
        }
    }
}

impl Default for SceneState {
    fn default() -> SceneState {
        let base = Arc::new(Transform::default());
        SceneState {
            width: 1024,
            height: 768
        }
    }
}

#[derive(Clone)]
pub struct SceneRef (Arc<RefCell<SceneState>>);

impl SceneRef {
    pub fn new(s: SceneState) -> SceneRef {
        SceneRef(Arc::new(RefCell::new(s)))
    }
}

impl std::ops::Deref for SceneRef {
    type Target = RefCell<SceneState>;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}


/*
 * A comma (potentially) surrounded by whitespace
 */
pub fn comma<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], (), E> 
where
    E: ParseError<&'a [u8]>
{
    use nom::character::complete::char;    
    ws(char(','))(input).map(|(i, _)| (i, ()))
}

pub fn ws<'a, T, E, ParserFn>(parser: ParserFn) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    E: ParseError<&'a [u8]>,
    ParserFn: Fn(&'a [u8]) -> IResult<&'a [u8], T, E>
{
    use nom::{
        sequence::delimited,
        character::streaming::multispace0
    };

    delimited(multispace0, parser, multispace0)
}

pub fn block<'a, T, Error, Parser>(parser: Parser) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>
where
    Parser: Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>,
    Error: ParseError<&'a [u8]>
{
    use nom::{
        sequence::delimited,
        character::streaming::char
    };

    let begin = ws(char('{'));
    let end = ws(char('}'));
    delimited(begin, parser, end)
}


pub fn named_object<'a, T, Error: ParseError<&'a [u8]>, ParserFn>(
        name: &'static str,
        parser: ParserFn) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>
where
    Error: ParseError<&'a [u8]>,
    ParserFn: Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>
{
    use nom::{
        sequence::{delimited, preceded},
        bytes::streaming::tag,
        character::{
            complete::multispace0,
            streaming::char
        }
    };

    ws(preceded(tag(name), parser))
}

pub fn named_value<'a, T, Error, ParserFn>(
    name: &'static str,
    parser: ParserFn) -> 
        impl Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>
where
    Error: ParseError<&'a [u8]>,
    ParserFn: Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>,
{
    use nom::{
        bytes::streaming::tag,
        character::{
            complete::multispace0,
            streaming::char
        }
    };

    move |input| {
        let (i,_) = multispace0(input)?;
        let (i,_) = tag(name)(i)?;
        let (i,_) = char(':')(i)?;
        let (i,_) = multispace0(i)?;
        let (i,v) = parser(i)?;
        let (i,_) = multispace0(i)?;

        Ok((i,v))
    }
}


pub fn map_named_value<'a, T, U, Error, ParserFn, MapFn>(
        name: &'static str,
        parser: ParserFn,
        mapfn: MapFn) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], U, Error>
where
    Error: ParseError<&'a [u8]>,
    ParserFn: Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>,
    MapFn: Fn(T) -> U
{
    use nom::combinator::map;
    map(named_value(name, parser), mapfn)
}

/*
 * A vector literal of the form {x, y, z}
 */

pub fn vector_literal(input: &[u8]) -> IResult<&[u8], Vector> {
    let (i, xx) = real_number(input)?;
    let (i, yy) = comma(i).and_then(|(i, _)| real_number(i))?;
    let (i, zz) = comma(i).and_then(|(i, _)| real_number(i))?;
    
    Ok((i, Vector::new(xx, yy, zz)))
}

// ////////////////////////////////////////////////////////////////////////////
// Parsing numbers
// ////////////////////////////////////////////////////////////////////////////

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

pub fn digit_string<'a>(input: &'a [u8]) -> IResult<&'a [u8], &'a str> {
    use std::str::from_utf8;
    use nom::{
        combinator::map,
        character::complete::digit0,
    };    
    map(digit0, |bs| from_utf8(bs).unwrap())(input)   
}

/*
 * Parses a real number represented as a decimal fraction (as opposed to one in
 * exponential notation)
 */
pub fn real_number(input: &[u8]) -> IResult<&[u8], f64>
{
    use std::str::from_utf8;
    use nom::{
        branch::alt,
        combinator::{map, opt},
        character::complete::{
            char,
            digit0,
            digit1,
        },
        sequence::preceded,
    };

    let as_str = |s| from_utf8(s).unwrap();

    let optional_sign = opt(alt((char('-'), char('+'))));
    let fraction = opt(preceded(char('.'), digit_string));
   
    let (i, sign) = optional_sign(input)?;
    let (i, int) = digit_string(i)?;
    let (i, frac) = fraction(i)?;

    Ok((i, to_real(sign, int, frac)))
}

pub fn as_object<PrimitiveT: Primitive>(p: PrimitiveT,
                                        m: Material,
                                        transform: Option<Transform>) -> Object {
    Object {
        primitive: Arc::new(p) as Arc<dyn Primitive>,
        material: m,
        transform: transform.map(|t| Box::new(t))
    }
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
                   IResult::Ok((&b""[..], ())));

        assert_eq!(f, 42.0);
    }

    #[test]
    fn parse_digit_string() {
        use nom::IResult;

        assert!(!digit_string(b"").is_ok(), "Empty string");
        assert!(!digit_string(b"abcd").is_ok(), "Text string");
        assert_eq!(digit_string(b"1234"), IResult::Ok((&b""[..], "1234")));
        assert_eq!(digit_string(b"1234a567"),
                   IResult::Ok((&b"a567"[..], "1234")));
    }

    #[test]
    fn parse_vector_literal() {
        use math::vector;
        use nom::IResult;

        let v = vector(1.0, 0.5, 0.0);
        let expected = IResult::Ok((&b""[..], v));

        assert_eq!(vector_literal(b"{1, 0.5, 0}"), expected);
        assert_eq!(vector_literal(b"{ 1.0 , 0.5, 0.0}"), expected);
        assert_eq!(vector_literal(b"{1.0,0.5,0.0 }"), expected);
    }

    #[test]
    fn parse_float() {
        use nom::IResult;

        assert!(real_number(b"").is_err(), "Empty string");
        assert_eq!(real_number(b"163"), IResult::Ok((&b""[..], 163.0)));
        assert_eq!(real_number(b"+163"), IResult::Ok((&b""[..], 163.0)));
        assert_eq!(real_number(b"-163"), IResult::Ok((&b""[..], -163.0)));
        assert_eq!(real_number(b"-163"), IResult::Ok((&b""[..], -163.0)));
        assert_eq!(real_number(b"27.01"), IResult::Ok((&b""[..], 27.01)));
        assert_eq!(real_number(b"-27.01"), IResult::Ok((&b""[..], -27.01)));

        assert_eq!(real_number(b"-12.34 plus some other text"),
                   IResult::Ok((&b" plus some other text"[..], -12.34)));

        assert_eq!(real_number(b"42 plus some other text"),
                   IResult::Ok((&b" plus some other text"[..], 42.0)));
    }
}
