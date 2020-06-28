
use nom::{
    IResult,
    bytes::complete::tag,
    sequence::preceded,
};

use crate::{
    constructs::*,
    colour::{colour}
};

use raygun_material::Pigment;

fn solid_pigment<'a>(scene: SceneRef) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Pigment> {
   
    let block_parser = block(
        map_named_value("colour", colour(scene), Pigment::Solid));

    preceded(ws(tag("solid")), block_parser)
}

pub fn pigment<'a>(scene: SceneRef) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], Pigment>
{        
    //ws(alt((solid_pigment, )))(input)
    ws(solid_pigment(scene))
}

#[cfg(test)]
mod test {
    use super::*;
    use raygun_material::Colour;

    #[test]
    fn solid_pigment() {
        let text = r#"solid {
            colour: { 0.1, 0.2, 0.3 }
        }"#;
        let scene = SceneRef::default();

        match pigment(scene)(text.as_bytes()) {
            IResult::Ok((_, p)) => {
                let Pigment::Solid(c) = p;
                assert_eq!(c, Colour::new(0.1, 0.2, 0.3))
            },
            IResult::Err(e) => assert!(false, "Parse failed: {:?}", e)
        }
    }
}
