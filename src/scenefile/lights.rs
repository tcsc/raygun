use nom::IResult;

use super::constructs::*;
use super::colour::*;

use colour::Colour;
use light::PointLight;
use material::Material;
use math::Matrix;
use primitive::Object;

pub fn point_light<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Object> {
    let mut result = PointLight::default();

    let rval = {
        named_object!(input,
                      "point_light",
                      block!(separated_list!(comma,
                                             alt!(call!(named_value,
                                                        "colour",
                                                        colour,
                                                        set!(result.colour)) |
                                                  call!(named_value,
                                                        "location",
                                                        vector_literal,
                                                        set!(result.loc))))))
    };

    rval.map(|_| as_object(result,
                           Material::default(),
                           Matrix::default()))
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

        match point_light(b"point_light { colour: {0.3, 0.4, 0.5}, location: {1, 2, 3} }",
                          &state) {
            IResult::Done(_, obj) => {
                let l = obj.as_primitive::<PointLight>().unwrap();
                assert_eq!(l.colour, Colour::new(0.3, 0.4, 0.5));
                assert_eq!(l.loc, point(1.0, 2.0, 3.0));
            }
            IResult::Error(_) |
            IResult::Incomplete(_) => assert!(false),
        }
    }
}
