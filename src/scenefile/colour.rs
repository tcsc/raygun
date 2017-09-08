use super::constructs::*;
use colour::Colour;

// ////////////////////////////////////////////////////////////////////////////
// Colours
// ////////////////////////////////////////////////////////////////////////////

/**
 * A colour literal of the form {r, g, b}
 */
named!(pub colour <Colour>, block!(
    do_parse!(rr: real_number >>
              comma >>
              gg: real_number >>
              comma >>
              bb: real_number >>
              (Colour::new(rr, gg, bb)))
));



// ////////////////////////////////////////////////////////////////////////
// Colour tests
// ////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_colour() {
        use colour::Colour;
        use nom::IResult;

        let c = Colour { r: 1.0, g: 0.5, b: 0.0 };
        let expected = IResult::Done(&b""[..], c);

        assert_eq!(colour(b"{1, 0.5, 0}"), expected);
        assert_eq!(colour(b"{ 1.0 , 0.5, 0.0}"), expected);
        assert_eq!(colour(b"{1.0,0.5,0.0 }"), expected);
    }
}