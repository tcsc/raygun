use super::{
    constructs::*,
    colour::*
};

use crate::{
    light::PointLight,
    material::Material,
    math::Transform,
    primitive::Object,
};

use nom::{
    error::ParseError,
    lib::std::ops::RangeFrom,
    AsChar, 
    InputIter, 
    Slice,
    IResult,
};

use std::cell::RefCell;

pub fn point_light(scene: SceneRef) -> 
    impl Fn(&[u8]) -> IResult<&[u8], Object>
{
    use nom::{
        branch::alt,
        multi::separated_list
    };

    move |input| {
        let mut result = PointLight::default();

        let p = named_object("point_light",
            block(separated_list(comma, 
                ws(alt((
                    named_value("colour", colour, |c| result.colour = c),
                    named_value("location", vector_literal, |l| result.loc = l)
                )))
            ))
        );
        
        p(input).map(|(i, _)| {
            (i, as_object(result, Material::default(), None))
        })
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_point_light() {
        use crate::{
            colour::Colour,
            math::point, 
            light::PointLight
        };
        use nom::IResult;

        let mut state = SceneState::default();

        match point_light(b"point_light { colour: {0.3, 0.4, 0.5}, location: {1, 2, 3} }",
                          &state) {
            IResult::Ok((_, obj)) => {
                let l = obj.as_primitive::<PointLight>().unwrap();
                assert_eq!(l.colour, Colour::new(0.3, 0.4, 0.5));
                assert_eq!(l.loc, point(1.0, 2.0, 3.0));
            }
            IResult::Err(_) => assert!(false),
        }
    }
}
