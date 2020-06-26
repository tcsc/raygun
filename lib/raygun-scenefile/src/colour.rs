use super::constructs::*;
use raygun_material::Colour; 

use nom::{
    error::ParseError,
    lib::std::ops::RangeFrom,
    IResult,
    AsChar,
    InputIter,
    Slice
};

// ////////////////////////////////////////////////////////////////////////////
// Colours
// ////////////////////////////////////////////////////////////////////////////

/*
 * A colour literal of the form {r, g, b}
 */
pub fn colour(input: &[u8]) -> IResult<&[u8], Colour> {
    let (i, rr) = real_number(input)?;
    let (i, gg) = comma(i).and_then(|(i, _)| real_number(i))?;
    let (i, bb) = comma(i).and_then(|(i, _)| real_number(i))?;

    Ok((i, Colour::new(rr, gg, bb)))
}

// ////////////////////////////////////////////////////////////////////////
// Colour tests
// ////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_colour() {
        use crate::colour::Colour;
        use nom::IResult;

        let c = Colour {
            r: 1.0,
            g: 0.5,
            b: 0.0,
        };
        let expected = IResult::Ok((&b""[..], c));

        assert_eq!(colour(b"{1, 0.5, 0}"), expected);
        assert_eq!(colour(b"{ 1.0 , 0.5, 0.0}"), expected);
        assert_eq!(colour(b"{1.0,0.5,0.0 }"), expected);
    }
}
