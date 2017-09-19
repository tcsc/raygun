use nom::IResult;

use colour::Colour;
use material::Material;
use math::Matrix;
use primitive::{Box as _Box, Object, Sphere};

use super::material::material;
use super::SceneState;
use super::constructs::*;
use super::lights::point_light;

fn sphere<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Object> {
    let mut result = Sphere::default();
    let mut m = Material::default();

    let rval = {
        named_object!(input, "sphere",
            block!(separated_list!(comma,
                alt!(
                    call!(named_value, "radius", real_number, set!(result.radius)) |
                    call!(named_value, "centre", vector_literal, set!(result.centre)) |
                    call!(named_value, "material", material, set!(m))
                )
            )
        ))
    };

    rval.map(|_| as_object(result, m, Matrix::default()))
}

fn _box<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Object> {
    let mut b = _Box::default();
    let mut m = Material::default();

    let rval = {
        named_object!(input, "box",
            block!(separated_list!(comma,
                ws!(alt!(
                    call!(named_value, "upper", vector_literal, set!(b.upper)) |
                    call!(named_value, "lower", vector_literal, set!(b.lower)) |
                    call!(named_value, "material", material, set!(m))
                )))
            ))
    };

    rval.map(|_| as_object(b, m, Matrix::default()))
}

pub fn primitive<'a>(input: &'a [u8], state: &SceneState) -> IResult<&'a [u8], Object> {
    alt!(input, call!(sphere, state) |
                call!(_box, state) |
                call!(point_light, state))
}


#[cfg(test)]
mod test {
    use super::*;
    use nom;

    #[test]
    fn parse_sphere() {
        use math::point;
        use primitive::Sphere;
        use nom::IResult;

        let state = SceneState::default();

        match sphere(b"sphere { radius: 1.2340, centre: {1, 2, 3} }", &state) {
            IResult::Done(_, obj) => {
                let s = obj.as_primitive::<Sphere>().unwrap();
                assert_eq!(s.radius, 1.234);
                assert_eq!(s.centre, point(1.0, 2.0, 3.0));
            }
            IResult::Error(_) |
            IResult::Incomplete(_) => assert!(false),
        }
    }

    #[test]
    fn parse_box() {
        use math::point;
        use primitive::Box as _Box;
        use nom::IResult;

        let state = SceneState::default();

        match _box(b"box { lower: {1,2,3}, upper: {4.1, 5.2, 6.3} }", &state) {
            IResult::Done(_, obj) => {
                let b = obj.as_primitive::<_Box>().unwrap();
                assert!(b.lower.approx_eq(point(1.0, 2.0, 3.0)),
                        "Actual: {:?}", b.lower);
                assert!(b.upper.approx_eq(point(4.1, 5.2, 6.3)));
            }
            IResult::Error(_) |
            IResult::Incomplete(_) => assert!(false),
        }
    }
}