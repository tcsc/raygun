use nom::IResult;

use material::Material;
use math::Matrix;
use primitive::{Box as _Box, Object, Plane, Sphere};

use super::material::material;
use super::SceneState;
use super::constructs::*;
use super::lights::point_light;

fn sphere<'a>(input: &'a [u8], scene: &mut SceneState) -> IResult<&'a [u8], Object> {
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

fn _box<'a>(input: &'a [u8], scene: &mut SceneState) -> IResult<&'a [u8], Object> {
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

fn plane<'a>(input: &'a [u8], scene: &SceneState) -> IResult<&'a [u8], Object> {
    let mut p = Plane::default();
    let mut m = Material::default();

    let rval = {
        named_object!(input, "plane",
            block!(separated_list!(comma,
                ws!(alt!(
                    call!(named_value, "normal", vector_literal,
                         |n| p.normal = n.normalize()) |
                    call!(named_value, "offset", real_number, set!(p.offset)) |
                    call!(named_value, "material", material, set!(m))
                )))
            ))
    };

    rval.map(|_| as_object(p, m, Matrix::default()))
}

pub fn primitive<'a>(input: &'a [u8], state: &mut SceneState) -> IResult<&'a [u8], Object> {
    alt!(input, call!(sphere, state) |
                call!(_box, state) |
                call!(plane, state) |
                call!(point_light, state))
}


#[cfg(test)]
mod test {
    use super::*;
    use nom;
    use math::vector;
    use float_cmp::ApproxEqUlps;

    #[test]
    fn parse_sphere() {
        use math::point;
        use primitive::Sphere;
        use nom::IResult;

        let mut state = SceneState::default();

        match sphere(b"sphere { radius: 1.2340, centre: {1, 2, 3} }", &mut state) {
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

        let mut state = SceneState::default();

        let (_, obj) = _box(b"box { lower: {1,2,3}, upper: {4.1, 5.2, 6.3} }",
                            &mut state).unwrap();

        let b = obj.as_primitive::<_Box>().unwrap();
        assert!(b.lower.approx_eq(point(1.0, 2.0, 3.0)),
                "Actual: {:?}", b.lower);
        assert!(b.upper.approx_eq(point(4.1, 5.2, 6.3)));
    }

    #[test]
    fn parse_plane() {
        use math::point;
        use primitive::Plane;
        use nom::IResult;

        let mut state = SceneState::default();

        let (_, obj) = plane(b"plane { normal: {1.2, 3.4, 5.6}, offset: 7.8 }",
                             &state)
            .unwrap();

        let p = obj.as_primitive::<Plane>().unwrap();
        let expected = vector(0.1801712440614613,
                              0.5104851915074736,
                              0.8407991389534859);

        assert!(p.normal.approx_eq(expected),
                "Expected normal {:?}, got {:?}", expected, p.normal);
        assert!(p.offset.approx_eq_ulps(&7.8, 1));
    }
}