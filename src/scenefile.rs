use std::f64;
use std::any::Any;
use std::collections::HashMap;
use std::str;
use std::str::FromStr;
// use pc::{Parser, ParserExt, parser, space, spaces, digit};
// use pc::primitives::{Consumed, ParseResult, State, ParseError, Stream};
// use pc::combinator::{Between, Token, FnParser, skip_many, many1, token, optional};
// use pc::char::{char};

use colour::Colour;
use math::{Vector, vector};
use primitive::{Primitive, Sphere};
use light::PointLight;

use nom::{multispace, digit};

named!(whitespace< Vec<char> >, many0!(one_of!(" \t")));

// ////////////////////////////////////////////////////////////////////////////
// Parsing numbers
// ////////////////////////////////////////////////////////////////////////////

named!(digit_string<&str>, map_res!(digit, str::from_utf8));

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
named!(vector_literal<Vector>, delimited!(
    preceded!(char!('{'), whitespace),
    chain!(
        xx: real_number ~
        whitespace ~ char!(',') ~ whitespace ~
        yy: real_number ~
        whitespace ~ char!(',') ~ whitespace ~
        zz: real_number,
        || { Vector::new(xx, yy, zz) }
    ),
    preceded!(whitespace, char!('}'))
));


/**
 * A colour literal of the form {r, g, b}
 */
named!(colour_literal<Colour>, delimited!(
    preceded!(char!('{'), whitespace),
    chain!(
        rr: real_number ~
        whitespace ~ char!(',') ~ whitespace ~
        gg: real_number ~
        whitespace ~ char!(',') ~ whitespace ~
        bb: real_number,
        || { Colour::new(rr, gg, bb) }
    ),
    preceded!(whitespace, char!('}'))
));

// ////////////////////////////////////////////////////////////////////////////
// A comma-separated list of N items
// ////////////////////////////////////////////////////////////////////////////


//fn list_of<T>(i :&[u8])

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
}

// impl<P> Parser for ListOf<P>
//     where P : Parser, <P as Parser>::Input : Stream<Item=char>
// {
//     type Input = <P as Parser>::Input;
//     type Output = Vec<<P as Parser>::Output>;

//     fn parse_state(&mut self, input: State<Self::Input>) ->
//         ParseResult<Self::Output, Self::Input>
//     {
//         let mut result = Vec::with_capacity(self.n);
//         let mut text = Consumed::Empty(input);

//         let (first, remainder) = try!(text.combine(|i| self.parser.parse_state(i)));
//         text = remainder;
//         result.push(first);

//         let sep = skip_many(space())
//                     .with(token::<Self::Input>(',' as <Self::Input as Stream>::Item))
//                     .skip(spaces());
//         for _ in 1 .. self.n {
//             let (x, rest) = try!(text.clone().combine(|i| {
//                 sep.clone().with(&mut self.parser).parse_state(i)
//             }));
//             result.push(x);
//             text = rest;
//         }

//         Ok((result, text))
//     }
// }

// ///
// /// Parses a comma-separated list of n tokens, where the token is defined by a
// /// user-supplied parser.
// ///
// fn list_of<P>(n: usize, p: P) -> ListOf<P>
//     where P: Parser
// {
//     ListOf { parser: p, n: n }
// }

// // ////////////////////////////////////////////////////////////////////////////
// // A named value
// // ////////////////////////////////////////////////////////////////////////////

// #[derive(Debug)]
// struct NamedValue {
//     name: String,
//     value: Box<Any>
// }

// struct Field<P> {
//     name: &'static str,
//     parser: P,
// }

// impl<P> Parser for Field<P> where P : Parser,
//     <P as Parser>::Input : Stream<Item=char>,
//     <P as Parser>::Output : Any
// {
//     type Input = <P as Parser>::Input;
//     type Output = NamedValue;

//     fn parse_state(&mut self, input: State<Self::Input>) ->
//            ParseResult<Self::Output, Self::Input> {
//         use pc::string;

//         let (val, remainder) =
//             try!(string(&self.name).with(spaces())
//                                    .with(token(':'))
//                                    .with(spaces())
//                                    .with(&mut self.parser)
//                                    .parse_state(input));
//         let field = NamedValue {
//             name: self.name.to_string(),
//             value: Box::new(val)
//         };

//         Ok((field, remainder))
//     }
// }

// fn field<P>(name: &'static str, parser: P) -> Field<P>
//     where P : Parser
// {
//     Field{ name: name, parser: parser }
// }

// // ////////////////////////////////////////////////////////////////////////////
// // Parses a list of fields
// // ////////////////////////////////////////////////////////////////////////////

// struct FieldList<P> { parser: P }

// impl<P> Parser for FieldList<P>
//     where P : Parser<Output=NamedValue>,
//          <P as Parser>::Input : Stream<Item=char>
// {
//     type Input = <P as Parser>::Input;
//     type Output = Vec<NamedValue>;

//     fn parse_state(&mut self, input: State<Self::Input>) ->
//             ParseResult<Self::Output, Self::Input> {
//         use pc::sep_by;
//         let arg_with_spaces = spaces().with(&mut self.parser).skip(spaces());
//         sep_by(arg_with_spaces, token(',')).parse_state(input)
//     }
// }

// ///
// /// Creates a field list parser around a set of options.
// ///
// fn field_list<P>(parser: P) -> FieldList<P>
//     where P : Parser<Output=NamedValue>,
//          <P as Parser>::Input : Stream<Item=char>
// {
//     FieldList { parser: parser }
// }

// ///
// /// Find the named field, returning the default value if it can't be found. If
// /// multiple instances of the field name are present, the first instance in the
// /// list is returned.
// ///
// fn find_arg<T: Any+Copy>(name: &str, argv: &Vec<NamedValue>, default: T) -> T {
//     match argv.iter().find(|x| x.name == name) {
//         Some(named_val) =>  {
//             *named_val.value.downcast_ref::<T>().unwrap()
//         },
//         None => default
//     }
// }

// // ////////////////////////////////////////////////////////////////////////////
// // A brace-delimited block
// // ////////////////////////////////////////////////////////////////////////////

// struct Block<P> {
//     parser: P
// }

// impl<P> Parser for Block<P>
//     where P : Parser, <P as Parser>::Input : Stream<Item=char>
// {
//     type Input = <P as Parser>::Input;
//     type Output = <P as Parser>::Output;

//     fn parse_state(&mut self, input: State<Self::Input>) ->
//             ParseResult<Self::Output, Self::Input> {
//         use pc::between;

//         let leader = token('{').with(spaces());
//         let footer = spaces().with(token('}'));
//         between(leader, footer, &mut self.parser)
//             .parse_state(input)
//     }
// }

// fn block<P>(parser: P) -> Block<P>
//     where P : Parser, <P as Parser>::Input : Stream<Item=char>
// {
//     Block { parser: parser }
// }

// // ////////////////////////////////////////////////////////////////////////////
// // A named, brace-delimited block
// // ////////////////////////////////////////////////////////////////////////////

// struct NamedBlock<P> {
//     name: &'static str,
//     parser: P
// }

// impl<P> Parser for NamedBlock<P>
//     where P : Parser, <P as Parser>::Input : Stream<Item=char>
// {
//     type Input = <P as Parser>::Input;
//     type Output = <P as Parser>::Output;

//     fn parse_state(&mut self, input: State<Self::Input>) ->
//             ParseResult<Self::Output, Self::Input> {
//         use pc::string;

//         string(self.name)
//             .skip(spaces())
//             .with(block(&mut self.parser))
//             .parse_state(input)
//     }
// }

// fn named_block<P>(name: &'static str, parser: P) -> NamedBlock<P>
//     where P : Parser, <P as Parser>::Input : Stream<Item=char>
// {
//     NamedBlock { name: name, parser: parser }
// }

// // ////////////////////////////////////////////////////////////////////////////
// // Parse state
// // ////////////////////////////////////////////////////////////////////////////

// struct SceneState {
//     colours: HashMap<String, Colour>
// }

// impl SceneState {
//     fn new() -> SceneState {
//         SceneState { colours: HashMap::new() }
//     }
// }

// // ////////////////////////////////////////////////////////////////////////////
// // Basic Parsing primitives
// // ////////////////////////////////////////////////////////////////////////////

// ///
// /// Parses a real number into a an f64
// ///
// fn real<I>(input: State<I>) -> ParseResult<f64, I>
//     where I: Stream<Item=char>
// {
//     let sign = optional(char('-').or(char('+')));
//     let integral = many1(digit());
//     let fraction = optional(char('.').and(many1(digit())));

//     sign.and(integral)
//         .and(fraction)
//         .map(|t: ((Option<char>, String), Option<(char, String)>)|{
//             let s = match (t.0).0 {
//                 Some('+') | None => 1.0,
//                 Some('-') => -1.0,
//                 Some(c) => panic!("Unexpected sign char: {:?}", c)
//             };

//             let i = (t.0).1.parse::<i64>().unwrap() as f64;

//             let f = match t.1 {
//                 None => 0.0,
//                 Some((_, digits)) => {
//                     let val = digits.parse::<i64>().unwrap() as f64;
//                     let scale = (10.0 as f64).powi(digits.len() as i32);
//                     val / scale
//                 }
//             };

//             (s * i) + (s * f)
//         })
//         .parse_state(input)
// }

// ///
// /// Parses a colour literal of the form { r, g, b } into a Colour object
// ///
// fn colour_literal<I>(input: State<I>) -> ParseResult<Colour, I>
//     where I: Stream<Item=char>
// {
//     block(list_of(3, parser(real)))
//         .map(|v: Vec<f64>| Colour {r: v[0], g: v[1], b: v[2]})
//         .parse_state(input)
// }

// fn vector_literal<I>(input: State<I>) -> ParseResult<Vector, I>
//     where I: Stream<Item=char>
// {
//     block(list_of(3, parser(real)))
//         .map(|v: Vec<f64>| vector(v[0], v[1], v[2]))
//         .parse_state(input)
// }

// // ////////////////////////////////////////////////////////////////////////////
// // Geometric Primitives
// // ////////////////////////////////////////////////////////////////////////////

// ///
// /// A sphere literal
// ///
// fn sphere<I>(input: State<I>) -> ParseResult<Box<Sphere>,I >
//     where I: Stream<Item=char>
// {
//     let args = field_list(
//             field("centre", parser(vector_literal))
//         .or(field("radius", parser(real)))
//     );

//     named_block("sphere", args).map(|argv: Vec<NamedValue>| {
//         let loc = find_arg("centre", &argv, vector(0.0, 0.0, 0.0));
//         let radius = find_arg("radius", &argv, 1.0 as f64);
//         Sphere::new(loc, radius)
//     })
//     .parse_state(input)
// }

// fn sphere_primitive<I>(input: State<I>) -> ParseResult<Box<Primitive>, I>
//     where I: Stream<Item=char>
// {
//     parser(sphere)
//         .map(|s| s as Box<Primitive>)
//         .parse_state(input)
// }

// ///
// /// A point light
// ///
// fn point_light<I>(input: State<I>) -> ParseResult<PointLight, I>
//     where I : Stream<Item=char>
// {
//     let args = field_list(
//             field("pos",    parser(vector_literal))
//         .or(field("colour", parser(colour_literal)))
//     );

//     named_block("point_light", args).map(|argv: Vec<NamedValue>| {
//         let pos =    find_arg("pos",    &argv, vector(0.0, 0.0, 0.0));
//         let colour = find_arg("colour", &argv, Colour::new(1.0, 1.0, 1.0));
//         PointLight::new(pos, colour)
//     })
//     .parse_state(input)
// }

// // ////////////////////////////////////////////////////////////////////////////
// //
// // ////////////////////////////////////////////////////////////////////////////

// #[cfg(test)]
// mod test {
//     use super::*;
//     use pc::{parser, Parser, ParseResult};
//     use pc::primitives::{State, Stream};

//     #[test]
//     fn parse_float() {
//         assert_eq!(parser(super::real).parse("0"), Ok((0.0, "")));
//         assert_eq!(parser(super::real).parse("163"), Ok((163.0, "")));
//         assert_eq!(parser(super::real).parse("123.456"), Ok((123.456, "")));
//         assert_eq!(parser(super::real).parse("-42.35"), Ok((-42.35, "")));

//         assert_eq!(parser(super::real).parse("-42.35 plus some other text..."),
//             Ok((-42.35, " plus some other text...")));
//     }

//     #[test]
//     fn list_parser_parses_list() {
//         use super::{real, list_of};
//         let rval = list_of(4, parser(real)).parse("1, 2 , 3,4");
//         let exp = vec!(1.0, 2.0, 3.0, 4.0);
//         assert_eq!(rval, Ok((exp, "")));
//     }

//     #[test]
//     fn named_value() {
//         use super::field;
//         use pc::hex_digit;

//         let rval = field("ten", hex_digit()).parse("ten: a");
//         if let Ok((x, _)) = rval {
//             assert_eq!(x.name, "ten");
//             assert_eq!(*x.value.downcast_ref::<char>().unwrap(), 'a')
//         }
//         else {
//             panic!("Got {:?}", rval)
//         }
//     }

//     #[test]
//     fn parse_field_list() {
//         use pc::{hex_digit, char, ParserExt};
//         use super::{field, field_list, real, find_arg};

//         let mut args = field_list(field("hex",  hex_digit())
//                               .or(field("real", parser(real)))
//                        );
//         match args.parse("hex: a, real: 3.14159") {
//             Ok((argv, _)) => {
//                 assert_eq!('a',     find_arg("hex",  &argv, '0'));
//                 assert_eq!(3.14159, find_arg("real", &argv, 0.0));
//             },
//             Err(e) => {
//                 panic!("{:?}", e);
//             }
//         }
//     }

//     #[test]
//     fn parse_vector_literal() {
//         use math::vector;

//         let col = || parser(super::vector_literal);
//         let expected = vector(1.0, 0.5, 0.0);

//         assert_eq!(col().parse("{1, 0.5, 0}"), Ok((expected, "")));
//         assert_eq!(col().parse("{ 1.0 , 0.5, 0.0}"), Ok((expected, "")));
//         assert_eq!(col().parse("{1.0,0.5,0.0 }"), Ok((expected, "")));
//     }

//     #[test]
//     fn parse_colour_literal() {
//         use colour::Colour;
//         let col = || parser(super::colour_literal);
//         let expected = Colour {r: 1.0, g: 0.5, b: 0.0};


//         assert_eq!(col().parse("{1, 0.5, 0}"),
//                    Ok((Colour{r: 1.0, g: 0.5, b: 0.0}, ""))
//         );

//         assert_eq!(col().parse("{ 1.0 , 0.5, 0.0}"),
//                    Ok((Colour{r: 1.0, g: 0.5, b: 0.0}, ""))
//         );


//         assert_eq!(col().parse("{1.0,0.5,0.0 }"),
//                    Ok((Colour{r: 1.0, g: 0.5, b: 0.0}, ""))
//         );
//     }

//     #[test]
//     fn parse_block() {
//         use super::block;
//         use pc::hex_digit;

//         let b = || block(hex_digit());

//         assert_eq!(b().parse("{a}"), Ok(('a', "")));
//         assert_eq!(b().parse("{ b}"), Ok(('b', "")));
//         assert_eq!(b().parse("{c }"), Ok(('c', "")));
//         assert_eq!(b().parse("{ d }"), Ok(('d', "")));
//     }

//     #[test]
//     fn parse_named_block() {
//         use super::named_block;
//         use pc::hex_digit;

//         let b = |s| named_block(s, hex_digit());

//         assert_eq!(b("first").parse("first {a}"), Ok(('a', "")));
//         assert_eq!(b("second").parse("second { b }"), Ok(('b', "")));
//         assert_eq!(b("third").parse("third   {c }"), Ok(('c', "")));
//         assert_eq!(b("fourth").parse("fourth{ d }"), Ok(('d', "")));
//     }

//     #[test]
//     fn parse_empty_sphere() {
//         use super::sphere;
//         use math::point;
//         use primitive::Sphere;

//         use std::any::Any;
//         let p = || parser(sphere);

//         match p().parse("sphere {}") {
//             Ok((obj, _)) => {
//                 assert_eq!(obj.radius(), 1.0);
//                 assert_eq!(obj.centre(), point(0.0, 0.0, 0.0))
//             },
//             Err(e) => panic!("Parse failed {:?}", e)
//         }
//     }

//     #[test]
//     fn parse_specified_sphere() {
//         use super::sphere;
//         use math::point;
//         use primitive::Sphere;

//         use std::any::Any;
//         let p = || parser(sphere);

//         match p().parse("sphere { centre: {0.0, 1.0, 2.0}, radius: 3.14159 }") {
//             Ok((obj, _)) => {
//                 assert_eq!(obj.radius(), 3.14159);
//                 assert_eq!(obj.centre(), point(0.0, 1.0, 2.0))
//             },
//             Err(e) => panic!("Parse failed {:?}", e)
//         }
//     }

//     #[test]
//     fn parse_point_light() {
//         use super::point_light;
//         use colour::Colour;
//         use math::point;
//         let p = || parser(point_light);

//         match p().parse("point_light {pos: {0.1, 2.3, 4.5}, colour: {6.7, 8.9, 10.11}}") {
//             Ok((obj, _)) => {
//                 assert_eq!(obj.position(), point(0.1, 2.3, 4.5));
//                 assert_eq!(obj.colour(), Colour::new(6.7, 8.9, 10.11));
//             },
//             Err(e) => panic!("Parse failed {:?}", e)
//         }
//     }
// }