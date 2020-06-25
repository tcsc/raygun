
use nom::{
    IResult,
    branch::alt
};

use super::super::{
    constructs::*,
    colour::colour
};

use crate::{
    colour::Colour,
    material::Pigment,
};

fn solid_pigment<'a>(input: &'a [u8]) -> IResult<&'a [u8], Pigment> {
    let mut result = Pigment::default();

    let rval = named_object("solid", 
        block(
            map_named_value("colour", colour, Pigment::Solid)
        )
    )(input);

    rval.map(|(i, _)| (i, result))
}

pub fn pigment<'a>(scene: SceneRef) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], Pigment>
{        
    //ws(alt((solid_pigment, )))(input)
    ws(solid_pigment)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn solid_pigment() {
        use crate::colour::Colour;

        let text = r#"solid {
            colour: { 0.1, 0.2, 0.3 }
        }"#;

        match pigment(text.as_bytes()) {
            IResult::Ok((_, p)) => {
                let Pigment::Solid(c) = p;
                assert_eq!(c, Colour::new(0.1, 0.2, 0.3))
            },
            IResult::Err(e) => assert!(false, "Parse failed: {:?}", e)
        }
    }
}
