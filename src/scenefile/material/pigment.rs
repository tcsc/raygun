
use nom::IResult;

use super::super::constructs::*;
use super::super::colour::colour;
use material::Pigment;

fn solid_pigment<'a>(input: &'a [u8]) -> IResult<&'a [u8], Pigment> {
    let mut result = Pigment::default();

    let rval = {
        named_object!(input,
                      "solid",
                      block!(call!(named_value,
                                   "colour",
                                   colour,
                                   |c| { result = Pigment::Solid(c); })))
    };

    rval.map(|_| result)
}

pub fn pigment<'a>(input: &'a [u8]) -> IResult<&'a [u8], Pigment> {
    ws!(input, alt!(solid_pigment))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn solid_pigment() {
        use colour::Colour;

        let text = r#"solid {
            colour: { 0.1, 0.2, 0.3 }
        }"#;

        match pigment(text.as_bytes()) {
            IResult::Done(_, p) => {
                let Pigment::Solid(c) = p;
                assert_eq!(c, Colour::new(0.1, 0.2, 0.3))
            },
            IResult::Error(e) => assert!(false, "Parse failed: {:?}", e),
            IResult::Incomplete(x) => assert!(false, "Parse incomplete: {:?}", x),
        }
    }
}
