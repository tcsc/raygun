use nom::IResult;

use material::Material;
use math::{self, Matrix, Vector};
use primitive::{Box as _Box, Object, Plane, Sphere};
use units::degrees;

use std::str;

use super::material::material;
use super::SceneState;
use super::constructs::*;
use super::lights::point_light;

fn sphere<'a, 'b>(input: &'a [u8], scene: &'b SceneState) -> IResult<&'a [u8], Object> {
    let mut result = Sphere::default();
    let mut m = Material::default();

    let rval = {
        named_object!(input, "sphere",
            block!(separated_list!(comma,
                ws!(alt!(
                    call!(named_value, "radius", real_number, set!(result.radius)) |
                    call!(named_value, "centre", vector_literal, set!(result.centre)) |
                    call!(named_value, "material", material, set!(m))
                ))
            )
        ))
    };

    rval.map(|_| {
        as_object(result, m, *scene.active_transform())
    }).map_err(|e| {
        println!("Sphere failed {:?}", e);
        e
    })
}

fn _box<'a, 'b>(input: &'a [u8], scene: &'b SceneState) -> IResult<&'a [u8], Object> {
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

    rval.map(|_| as_object(b, m, *scene.active_transform()))
        .map_err(|e| {
            println!("Box failed {:?}", e);
            e
        })
}

fn plane<'a, 'b>(input: &'a [u8], scene: &'b SceneState) -> IResult<&'a [u8], Object> {
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

    rval.map(|_| as_object(p, m, *scene.active_transform()))
        .map_err(|e| {
            println!("Plane failed {:?}", e);
            e
        })
}

///
/// Parses a group of objects, arbitrarily transformed. Transforms are applied in the order
/// they're encountered, and nested groups are allowed.
///
fn group<'a, 'b>(input: &'a [u8], state: &'b mut SceneState) -> IResult<&'a [u8], Vec<Object>> {
    use std::cell::RefCell;

    println!("Trying group...");

    let mut transform = math::IDENTITY;
    let mut result = Vec::new();

    let rval = {
        named_object!(input, "group",
            block!(separated_list!(comma,
                ws!(alt!(
                    call!(named_value, "translate", vector_literal,
                         |Vector {x, y, z}| {
                            transform = transform.translate(x, y, z);
                         }) |
                    call!(named_value, "rotate", vector_literal,
                          |Vector {x, y, z}| {
                            transform = transform.rotate(degrees(x).radians(),
                                                         degrees(y).radians(),
                                                         degrees(z).radians());
                          }) |
                    call!(named_value, "scale", vector_literal,
                          |Vector {x, y, z}| {
                            transform = transform.scale(x, y, z);
                          }) |
                    call!(named_value, "objects",
                          |i| {
                            state.push_transform(transform);
                            println!("parsing children...");
                            let rval = ws!(i, block!(call!(primitives, state)));
                            state.pop_transform();
                            rval
                          },
                          set!(result))
                ))
            ))
        )
    };

    rval.map(|_| result)
        .map_err(|e| {
            println!("Group failed {:?}", e);
            e
        })
}

pub fn primitive<'a, 'b>(input: &'a[u8], state: &'b SceneState) -> IResult<&'a [u8], Object> {
    ws!(input,
        alt!(
            call!(sphere, state) |
            call!(_box, state) |
            call!(plane, state) |
            call!(point_light, state)
        )
    )
}

pub fn primitives<'a, 'b>(input: &'a [u8], state: &'b mut SceneState)
    -> IResult<&'a [u8], Vec<Object>>
{
    use nom::ErrorKind;

    let mut result = Vec::new();
    let mut i = input;
    while i.len() > 0 {
        let g = ws!(i, map!(call!(group, state),
                            |mut os| result.append(&mut os)));
        let p = ws!(i, map!(call!(primitive, state),
                            |o| result.push(o)));

        match p.or(g) {
            IResult::Done(new_i, _) => {
                if new_i == i {
                    // we didn't consume anything. If we don't bail out now then
                    // we'll end up in an infinite loop next time around. Better
                    // to bail out now.
                    return IResult::Error(error_position!(ErrorKind::Many0, i));
                }
                i = new_i;
            },
            IResult::Error(_) => {
                return IResult::Done(i, result);
            },
            IResult::Incomplete(n) => {
                return IResult::Incomplete(n)
            }
        }
    }

    IResult::Done(i, result)
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

        let state = SceneState::default();
        let (_, obj) = sphere(
            b"sphere { radius: 1.2340, centre: {1, 2, 3} }",
            &state).unwrap();

        let s = obj.as_primitive::<Sphere>().unwrap();
        assert_eq!(s.radius, 1.234);
        assert_eq!(s.centre, point(1.0, 2.0, 3.0));
    }

    #[test]
    fn parse_sphere_default() {
        use math::point;
        use primitive::Sphere;
        use nom::IResult;

        let state = SceneState::default();
        let (_, obj) = sphere(b"sphere { }", &state).unwrap();

        let s = obj.as_primitive::<Sphere>().unwrap();
        assert_eq!(s.radius, 1.0);
        assert_eq!(s.centre, point(0.0, 0.0, 0.0));
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

    #[test]
    fn parse_group() {
        let text = r#"group {
            translate: {1, 2, 3},
            rotate: {90, 45, 180},
            scale: {0.5, 2.0, 1.5},

            objects: {
                sphere { }
                box {}
                plane {}
            }
        }"#;

        let mut state = SceneState::default();
        let (_, v) = group(text.as_bytes(), &mut state).unwrap();

        assert_eq!(v.len(), 3);

        let expected_transform = math::IDENTITY
            .translate(1.0, 2.0, 3.0)
            .rotate(degrees(90.0).radians(),
                    degrees(45.0).radians(),
                    degrees(180.0).radians())
            .scale(0.5, 2.0, 1.5);

        for o in v.iter() {
            assert_eq!(*o.transform(), expected_transform);
        }
    }

    #[test]
    fn parse_nested_group() {
        let text = r#"group {
            translate: {1, 2, 3},
            objects: {
                sphere {}
                group {
                    translate: {4, 5, 6},
                    objects: { box { } }
                }
                box {}
            }
        }"#;

        let mut state = SceneState::default();
        let (_, v) = group(text.as_bytes(), &mut state)
            .map_err(|e| {
                println!("Fail: {:?}", e);
                e
            }).unwrap();
        let base = math::IDENTITY.translate(1.0, 2.0, 3.0);

        assert_eq!(3, v.len());
        assert_eq!(*v[0].transform(), base);
        assert_eq!(*v[1].transform(), base.translate(4.0, 5.0, 6.0));
        assert_eq!(*v[2].transform(), base);

    }
}