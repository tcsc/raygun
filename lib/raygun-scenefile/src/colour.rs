use super::constructs::*;
use raygun_material::Colour;

use nom::{
    combinator::map,
    sequence::{terminated, tuple},
    IResult,
};

// ////////////////////////////////////////////////////////////////////////////
// Colours
// ////////////////////////////////////////////////////////////////////////////

pub fn colour<'a>(_scene: SceneRef) -> impl Fn(&'a [u8]) -> IResult<&[u8], Colour> {
    colour_literal
}

/*
 * A colour literal of the form {r, g, b}
 */
pub fn colour_literal(input: &[u8]) -> IResult<&[u8], Colour> {
    let parse_r = terminated(real_number, comma);
    let parse_g = terminated(real_number, comma);
    let parse_b = real_number;
    let parse_vector = block(tuple((parse_r, parse_g, parse_b)));

    map(parse_vector, |(r, g, b)| Colour::new(r, g, b))(input)
}

// ////////////////////////////////////////////////////////////////////////
// Colour tests
// ////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
    use raygun_material::Colour;

    macro_rules! colour_literal_tests {
        ($($name:ident: $text:expr, $expected:expr, $remainder:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let expected = Ok(($remainder.as_bytes(), $expected));
                    let actual = colour_literal($text.as_bytes());
                    assert_eq!(expected, actual);
                }
            )*
        }
    }

    colour_literal_tests! {
        literal_packed: "{1,0.5,0}", Colour {r: 1.0, g: 0.5, b: 0.0}, "",
        literal_spaced: "{ 1.0 , 0.5, 0.0}", Colour {r: 1.0, g: 0.5, b: 0.0}, "",
        literal_extra_spaced: "{ 1.0 , 0.5, 0.0}", Colour {r: 1.0, g: 0.5, b: 0.0}, "",
    }
}
