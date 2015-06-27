use std::f64;

use pc::{Parser, ParserExt, parser, space, spaces, digit};
use pc::primitives::{Consumed, ParseResult, State, Error, Stream};
use pc::combinator::{Between, Token, FnParser, skip_many, many1, token, optional};
use pc::char::{char};

use colour::Colour;

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

///
/// Parses a colour literal of the form { r, g, b } into a Colour object
///
fn colour_literal<I>(input: State<I>) -> ParseResult<Colour, I, char>
    where I: Stream<Item=char>
{
    use pc::between;

    between(token('{').with(spaces()),
            spaces().with(token('}')),
            list_of(3, parser(real)))
        .map(|v: Vec<f64>| Colour {r: v[0], g: v[1], b: v[2]})
        .parse_state(input)
}

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
}