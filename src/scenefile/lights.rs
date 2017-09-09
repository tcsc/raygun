use nom::{IResult};

use super::constructs::*;
use super::colour::*;

use colour::{Colour};
use light::PointLight;

pub fn point_light<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Box<PointLight>> {
    let mut result = PointLight::default();

    let rval = {
        named_object!(input, "point_light",
            block!(separated_list!(comma,
                alt!(
                    call!(named_value, "colour", |i| {colour(i)}, |c| { result.colour = c;}) |
                    call!(named_value, "location", vector_literal, |p| { result.loc = p; })
                )
            )))
    };

    match rval {
        IResult::Done(i, _) => IResult::Done(i, Box::new(result)),
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_point_light() {
        use colour::Colour;
        use math::point;
        use light::PointLight;
        use nom::IResult;

        let mut state = SceneState::default();

        match point_light(b"point_light { colour: {0.3, 0.4, 0.5}, location: {1, 2, 3} }", &state) {
            IResult::Done(_, l) => {
                assert_eq!(l.colour, Colour::new(0.3, 0.4, 0.5));
                assert_eq!(l.loc, point(1.0, 2.0, 3.0));
            },
            IResult::Error(_) | IResult::Incomplete(_) => assert!(false)
        }
    }
}