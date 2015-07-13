use std::f64;
use std::any::Any;

use pc::{Parser, ParserExt, parser, space, spaces, digit};
use pc::primitives::{Consumed, ParseResult, State, Error, Stream};
use pc::combinator::{Between, Token, FnParser, skip_many, many1, token, optional};
use pc::char::{char};

use colour::Colour;
use math::{Vector, vector};
use primitive::{Primitive, Sphere};

// ////////////////////////////////////////////////////////////////////////////
// A list of N items
// ////////////////////////////////////////////////////////////////////////////

struct ListOf<P> {
    parser: P,
    n: usize
}

impl<P> Parser for ListOf<P>
    where P : Parser, <P as Parser>::Input : Stream<Item=char>
{
    type Input = <P as Parser>::Input;
    type Output = Vec<<P as Parser>::Output>;

    fn parse_state(&mut self, input: State<Self::Input>) ->
        ParseResult<Self::Output, Self::Input, <Self::Input as Stream>::Item>
    {
        let mut result = Vec::with_capacity(self.n);
        let mut text = Consumed::Empty(input);

        let (first, remainder) = try!(text.combine(|i| self.parser.parse_state(i)));
        text = remainder;
        result.push(first);

        let sep = skip_many(space())
                    .with(token::<Self::Input>(',' as <Self::Input as Stream>::Item))
                    .skip(spaces());
        for _ in 1 .. self.n {
            let (x, rest) = try!(text.clone().combine(|i| {
                sep.clone().with(&mut self.parser).parse_state(i)
            }));
            result.push(x);
            text = rest;
        }

        Ok((result, text))
    }
}

///
/// Parses a comma-separated list of tokens, where the token is defined by a
/// user-supplied parser.
///
fn list_of<P>(n: usize, p: P) -> ListOf<P>
    where P: Parser
{
    ListOf { parser: p, n: n }
}

// ////////////////////////////////////////////////////////////////////////////
// A named value
// ////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct NamedValue {
    name: String,
    value: Box<Any>
}

struct Field<P> {
    name: &'static str,
    parser: P,
}

impl<P> Parser for Field<P>
    where P : Parser,
        <P as Parser>::Input : Stream<Item=char>,
        <P as Parser>::Output : Any
{
    type Input = <P as Parser>::Input;
    type Output = NamedValue;

    fn parse_state(&mut self, input: State<Self::Input>) ->
        ParseResult<NamedValue, Self::Input, <Self::Input as Stream>::Item>
    {
        use pc::string;

        let (val, remainder) =
            try!(string(&self.name).with(spaces())
                                   .with(token(':'))
                                   .with(spaces())
                                   .with(&mut self.parser)
                                   .parse_state(input));
        let field = NamedValue {
            name: self.name.to_string(),
            value: Box::new(val)
        };

        Ok((field, remainder))
    }
}

fn field<P>(name: &'static str, parser: P) -> Field<P>
    where P : Parser
{
    Field{ name: name, parser: parser }
}

// ////////////////////////////////////////////////////////////////////////////
// A brace-delimited block
// ////////////////////////////////////////////////////////////////////////////

struct Block<P> {
    parser: P
}

impl<P> Parser for Block<P>
    where P : Parser, <P as Parser>::Input : Stream<Item=char>
{
    type Input = <P as Parser>::Input;
    type Output = <P as Parser>::Output;

    fn parse_state(&mut self, input: State<Self::Input>) ->
        ParseResult<Self::Output, Self::Input, char>
    {
        use pc::between;

        let leader = token('{').with(spaces());
        let footer = spaces().with(token('}'));
        between(leader, footer, &mut self.parser)
            .parse_state(input)
    }
}

fn block<P>(parser: P) -> Block<P>
    where P : Parser, <P as Parser>::Input : Stream<Item=char>
{
    Block { parser: parser }
}

// ////////////////////////////////////////////////////////////////////////////
// A named, brace-delimited block
// ////////////////////////////////////////////////////////////////////////////

struct NamedBlock<P> {
    name: &'static str,
    parser: P
}

impl<P> Parser for NamedBlock<P>
    where P : Parser, <P as Parser>::Input : Stream<Item=char>
{
    type Input = <P as Parser>::Input;
    type Output = <P as Parser>::Output;

    fn parse_state(&mut self, input: State<Self::Input>) ->
        ParseResult<Self::Output, Self::Input, char>
    {
        use pc::string;

        string(self.name)
            .skip(spaces())
            .with(block(&mut self.parser))
            .parse_state(input)
    }
}

fn named_block<P>(name: &'static str, parser: P) -> NamedBlock<P>
    where P : Parser, <P as Parser>::Input : Stream<Item=char>
{
    NamedBlock { name: name, parser: parser }
}

// ////////////////////////////////////////////////////////////////////////////
// Basic Parsing primitives
// ////////////////////////////////////////////////////////////////////////////

///
/// Parses a real number into a an f64
///
fn real<I>(input: State<I>) -> ParseResult<f64, I, char>
    where I: Stream<Item=char>
{
    let sign = optional(char('-').or(char('+')));
    let integral = many1(digit());
    let fraction = optional(char('.').and(many1(digit())));

    sign.and(integral)
        .and(fraction)
        .map(|t: ((Option<char>, String), Option<(char, String)>)|{
            let s = match (t.0).0 {
                Some('+') | None => 1.0,
                Some('-') => -1.0,
                Some(c) => panic!("Unexpected sign char: {:?}", c)
            };

            let i = (t.0).1.parse::<i64>().unwrap() as f64;

            let f = match t.1 {
                None => 0.0,
                Some((_, digits)) => {
                    let val = (digits.parse::<i64>().unwrap() as f64);
                    let scale = (10.0 as f64).powi(digits.len() as i32);
                    val / scale
                }
            };

            (s * i) + (s * f)
        })
        .parse_state(input)
}

///
/// Parses a colour literal of the form { r, g, b } into a Colour object
///
fn colour_literal<I>(input: State<I>) -> ParseResult<Colour, I, char>
    where I: Stream<Item=char>
{
    block(list_of(3, parser(real)))
        .map(|v: Vec<f64>| Colour {r: v[0], g: v[1], b: v[2]})
        .parse_state(input)
}

fn vector_literal<I>(input: State<I>) -> ParseResult<Vector, I, char>
    where I: Stream<Item=char>
{
    block(list_of(3, parser(real)))
        .map(|v: Vec<f64>| vector(v[0], v[1], v[2]))
        .parse_state(input)
}

// ////////////////////////////////////////////////////////////////////////////
// Geometric Primitives
// ////////////////////////////////////////////////////////////////////////////

// fn sphere<I>(input: State<I>) -> ParseResult<Sphere, I, char> {
//     use pc::string;
//     use pc::between;

//     ler arg =

//     let body = between(token)

//     string("sphere").skip(spaces).with(

//         )
// }

// ////////////////////////////////////////////////////////////////////////////
//
// ////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
    use pc::{parser, Parser, ParseResult};
    use pc::primitives::{State, Stream};

    #[test]
    fn parse_float() {
        assert_eq!(parser(super::real).parse("0"), Ok((0.0, "")));
        assert_eq!(parser(super::real).parse("163"), Ok((163.0, "")));
        assert_eq!(parser(super::real).parse("123.456"), Ok((123.456, "")));
        assert_eq!(parser(super::real).parse("-42.35"), Ok((-42.35, "")));

        assert_eq!(parser(super::real).parse("-42.35 plus some other text..."),
            Ok((-42.35, " plus some other text...")));
    }

    #[test]
    fn list_parser_parses_list() {
        use super::{real, list_of};
        let rval = list_of(4, parser(real)).parse("1, 2 , 3,4");
        let exp = vec!(1.0, 2.0, 3.0, 4.0);
        assert_eq!(rval, Ok((exp, "")));
    }

    #[test]
    fn named_value() {
        use super::field;
        use pc::hex_digit;

        let rval = field("ten", hex_digit()).parse("ten: a");
        if let Ok((x, _)) = rval {
            assert_eq!(x.name, "ten");
            assert_eq!(*x.value.downcast_ref::<char>().unwrap(), 'a')
        }
        else {
            panic!("Got {:?}", rval)
        }
    }

    #[test]
    fn parse_vector_literal() {
        use math::vector;

        let col = || parser(super::vector_literal);
        let expected = vector(1.0, 0.5, 0.0);

        assert_eq!(col().parse("{1, 0.5, 0}"), Ok((expected, "")));
        assert_eq!(col().parse("{ 1.0 , 0.5, 0.0}"), Ok((expected, "")));
        assert_eq!(col().parse("{1.0,0.5,0.0 }"), Ok((expected, "")));
    }

    #[test]
    fn parse_colour_literal() {
        use colour::Colour;
        let col = || parser(super::colour_literal);
        let expected = Colour {r: 1.0, g: 0.5, b: 0.0};


        assert_eq!(col().parse("{1, 0.5, 0}"),
                   Ok((Colour{r: 1.0, g: 0.5, b: 0.0}, ""))
        );

        assert_eq!(col().parse("{ 1.0 , 0.5, 0.0}"),
                   Ok((Colour{r: 1.0, g: 0.5, b: 0.0}, ""))
        );


        assert_eq!(col().parse("{1.0,0.5,0.0 }"),
                   Ok((Colour{r: 1.0, g: 0.5, b: 0.0}, ""))
        );
    }

    #[test]
    fn parse_block() {
        use super::block;
        use pc::hex_digit;

        let b = || block(hex_digit());

        assert_eq!(b().parse("{a}"), Ok(('a', "")));
        assert_eq!(b().parse("{ b}"), Ok(('b', "")));
        assert_eq!(b().parse("{c }"), Ok(('c', "")));
        assert_eq!(b().parse("{ d }"), Ok(('d', "")));
    }

    #[test]
    fn parse_named_block() {
        use super::named_block;
        use pc::hex_digit;

        let b = |s| named_block(s, hex_digit());

        assert_eq!(b("first").parse("first {a}"), Ok(('a', "")));
        assert_eq!(b("second").parse("second { b }"), Ok(('b', "")));
        assert_eq!(b("third").parse("third   {c }"), Ok(('c', "")));
        assert_eq!(b("fourth").parse("fourth{ d }"), Ok(('d', "")));
    }
}