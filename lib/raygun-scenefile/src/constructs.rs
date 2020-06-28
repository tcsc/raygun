use std::sync::Arc;
use std::cell::RefCell;

use nom::{
    bytes::complete::tag,
    character::complete::{ char as _char, multispace0 },
    combinator::{map, value},
    error::ParseError,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};  

use raygun_material::Material;
use raygun_primitives::{Object, Primitive};
use raygun_math::{Vector, Transform};

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
        let _base = Arc::new(Transform::default());
        SceneState {
            width: width,
            height: height
        }
    }
}

impl Default for SceneState {
    fn default() -> SceneState {
        let _base = Arc::new(Transform::default());
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

    #[cfg(test)]
    pub fn default() -> SceneRef {
        SceneRef(Arc::new(RefCell::new(SceneState::default())))
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
    value((), ws(_char(',')))(input)
}

pub fn ws<'a, T, E, ParserFn>(parser: ParserFn) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    E: ParseError<&'a [u8]>,
    ParserFn: Fn(&'a [u8]) -> IResult<&'a [u8], T, E>
{
    delimited(multispace0, parser, multispace0)
}

pub fn block<'a, T, Error, Parser>(parser: Parser) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>
where
    Parser: Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>,
    Error: ParseError<&'a [u8]>
{
    let begin = ws(_char('{'));
    let end = ws(_char('}'));
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
    ws(preceded(ws(tag(name)), parser))
}

pub fn named_value<'a, T, Error, ParserFn>(
    name: &'static str,
    parser: ParserFn) -> 
        impl Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>
where
    Error: ParseError<&'a [u8]>,
    ParserFn: Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>,
{
    move |input| {
        let (i,_) = multispace0(input)?;
        let (i,_) = tag(name)(i)?;
        let (i,_) = _char(':')(i)?;
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
    map(named_value(name, parser), mapfn)
}

/*
 * A vector literal of the form {x, y, z}
 */
pub fn vector_literal(input: &[u8]) -> IResult<&[u8], Vector> {
    let parse_x = terminated(real_number, comma);
    let parse_y = terminated(real_number, comma);
    let parse_z = real_number;
    let parse_vector = block(tuple((parse_x, parse_y, parse_z)));

    map(parse_vector, |(x, y, z)| Vector::new(x, y, z))(input)
}

// ////////////////////////////////////////////////////////////////////////////
// Parsing numbers
// ////////////////////////////////////////////////////////////////////////////

fn trace<T, E: std::fmt::Debug>(e: (T, E)) -> (T, E) 
{
    println!("Parse failure: {:?}", e.1);
    e
}

/*
 * Parses a real number represented as a decimal fraction (as opposed to one in
 * exponential notation)
 */
pub fn real_number(input: &[u8]) -> IResult<&[u8], f64>
{   
    nom::number::complete::double(input)
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
    use nom::error::ErrorKind;
    use raygun_math::vector;

    macro_rules! comma_tests {
        ($($name:ident: $text:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let expected = Ok(("".as_bytes(), ()));
                    let actual = comma::<(&[u8], ErrorKind)>($text.as_bytes());
                    assert_eq!(expected, actual);
                }
            )*
        }
    }

    comma_tests!{
        comma_bare: ",",
        comma_leading_whitespace: " ,",
        comma_trailing_whitespace: ", ",
        comma_surrounding_whitespace: " , ",
        comma_extended_whitespace: "\n,    ",
    }

    #[test]
    fn parse_named_value() {
        let result = named_value("float",  real_number)(b"float: 42");
        assert_eq!(result, Ok((&b""[..], 42.0)));
    }

    macro_rules! vector_literal_tests {
        ($($name:ident: $text:expr, $expected:expr, $remainder:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let expected = Ok(($remainder.as_bytes(), $expected));
                    let actual = vector_literal($text.as_bytes());
                    assert_eq!(expected, actual);
                }
            )*
        }
    }

    vector_literal_tests!{
        vector_packed: "{1,0.5, 0}", vector(1.0, 0.5, 0.0), "",
        vector_trailing_spaces: "{1, 0.5, 0}", vector(1.0, 0.5, 0.0), "",
        vector_extra_spaces: "{ 1.0 , 0.5, 0.0 }", vector(1.0, 0.5, 0.0), "",
    }

    macro_rules! float_tests {
        ($($name:ident: $text:expr, $expected:expr, $remainder:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let expected = Ok(($remainder.as_bytes(), $expected));
                    let actual = real_number($text.as_bytes());
                    assert_eq!(expected, actual);
                }
            )*
        }
    }

    float_tests! {
        integer_bare: "163", 163.0, "", 
        float_explicit_positive_int: "+163", 163.0, "",
        float_explicit_negative_int: "-163.0", -163.0, "",
        float_bare_decimal: "27.01", 27.01, "",
        float_explicit_positive_decimal: "+27.01", 27.01, "",
        float_explicit_negative_decimal: "-27.01", -27.01, "",
        float_with_trailing_text: "-12.34 plus some other text", -12.34, 
            " plus some other text",
        integer_with_trailing_text: "42 plus some other text", 42.0,
            " plus some other text",
    }
}
