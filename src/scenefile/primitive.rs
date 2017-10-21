use nom::IResult;

use material::Material;
use math::{self, Vector, Transform};
use primitive::{AxisAlignedBox, Box as _Box, Object, Plane, Sphere, Union};
use units::degrees;

use std::str;
use std::sync::Arc;

use super::material::material;
use super::SceneState;
use super::constructs::*;
use super::lights::point_light;
use super::transform::*;

fn sphere<'a, 'b>(input: &'a [u8], scene: &'b SceneState) -> IResult<&'a [u8], Object> {
    let mut result = Sphere::default();
    let mut m = Material::default();
    let mut xform = None;

    let rval = {
        named_object!(input, "sphere",
            block!(separated_list!(comma,
                ws!(alt!(
                    call!(named_value, "radius", real_number, set!(result.radius)) |
                    call!(named_value, "centre", vector_literal, set!(result.centre)) |
                    call!(named_value, "material", material, set!(m)) |
                    call!(named_value, "transform", transform, |t| xform = Some(t))
                ))
            )
        ))
    };

    rval.map(|_| {
        as_object(result, m, xform)
    })
}

fn _box<'a, 'b>(input: &'a [u8], scene: &'b SceneState) -> IResult<&'a [u8], Object> {
    let mut b = AxisAlignedBox::default();
    let mut m = Material::default();
    let mut xform = None;

    let rval = {
        named_object!(input, "box",
            block!(separated_list!(comma,
                ws!(alt!(
                    call!(named_value, "upper", vector_literal, set!(b.upper)) |
                    call!(named_value, "lower", vector_literal, set!(b.lower)) |
                    call!(named_value, "material", material, set!(m)) |
                    call!(named_value, "transform", transform, |t| xform = Some(t))
                )))
            ))
    };

    rval.map(|_| as_object(_Box::from(b), m, xform))
}

fn plane<'a, 'b>(input: &'a [u8], scene: &'b SceneState) -> IResult<&'a [u8], Object> {
    let mut p = Plane::default();
    let mut m = Material::default();
    let mut xform = None;

    let rval = {
        named_object!(input, "plane",
            block!(separated_list!(comma,
                ws!(alt!(
                    call!(named_value, "normal", vector_literal,
                         |n| p.normal = n.normalize()) |
                    call!(named_value, "offset", real_number, set!(p.offset)) |
                    call!(named_value, "material", material, set!(m)) |
                    call!(named_value, "transform", transform, |t| xform = Some(t))
                )))
            ))
    };

    rval.map(|_| as_object(p, m, xform))
}

///
/// Parses a group of objects, arbitrarily transformed. Transforms are applied in the order
/// they're encountered, and nested groups are allowed.
///
fn union<'a, 'b>(input: &'a [u8], state: &'b mut SceneState) -> IResult<&'a [u8], Object> {
    use std::cell::RefCell;

    let mut u = Union::default();
    let mut xform = None;

    let rval = {
        named_object!(input, "union",
            block!(separated_list!(comma,
                ws!(alt!(
                    call!(named_value, "transform", transform, |t| xform = Some(t)) |
                    call!(named_value, "objects",
                          |i| {
                            info!("parsing children...");
                            let rval = ws!(i, block!(call!(primitives, state)));
                            rval
                          },
                          set!(u.children))
                ))
            ))
        )
    };

    rval.map(|_| as_object(u, Material::default(), xform))
}

pub fn primitive<'a, 'b>(input: &'a[u8], state: &'b mut SceneState) -> IResult<&'a [u8], Object> {
    ws!(input,
        alt!(
            call!(sphere, state) |
            call!(_box, state) |
            call!(plane, state) |
            call!(point_light, state) |
            call!(union, state)
        )
    )
}

pub fn primitives<'a, 'b>(input: &'a [u8], state: &'b mut SceneState)
    -> IResult<&'a [u8], Vec<Arc<Object>>>
{
    many0!(input, map!(call!(primitive, state), |p| Arc::new(p)))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ops::Deref;
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
        assert!(b.lower().approx_eq(point(1.0, 2.0, 3.0)),
                "Actual: {:?}", b.lower());
        assert!(b.upper().approx_eq(point(4.1, 5.2, 6.3)));
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
    fn parse_union() {
        let text = r#"union {
            transform: {
                translate: {1, 2, 3},
                rotate: {90, 45, 180},
                scale: {0.5, 2.0, 1.5}
            },

            objects: {
                sphere { }
                box {}
                plane {}
            }
        }"#;

        let mut state = SceneState::default();
        let (_, obj) = union(text.as_bytes(), &mut state).unwrap();

        let expected_transform = Transform::for_translation(1.0, 2.0, 3.0)
            .rotate(degrees(90.0).radians(),
                    degrees(45.0).radians(),
                    degrees(180.0).radians())
            .scale(0.5, 2.0, 1.5);
        assert_eq!(obj.transform.as_ref().unwrap().deref(),
                   &expected_transform);

        let u = obj.as_primitive::<Union>().unwrap();
        assert_eq!(u.children.len(), 3);
    }

    #[test]
    fn parse_nested_group() {
        let text = r#"union {
            transform: {
              translate: {1, 2, 3}
            },
            objects: {
                sphere {}
                union {
                    transform: {
                        translate: {4, 5, 6}
                    },
                    objects: { box { } }
                }
                box {}
            }
        }"#;

        let mut state = SceneState::default();
        let (_, obj) = union(text.as_bytes(), &mut state).unwrap();
        let base = Transform::for_translation(1.0, 2.0, 3.0);

        let u = obj.as_primitive::<Union>().unwrap();

        assert_eq!(3, u.children.len());
        assert_eq!(base, *obj.transform.as_ref().unwrap().deref());

        let nested = &u.children[1];
        let nested_union = nested.as_primitive::<Union>().unwrap();
        assert_eq!(1, nested_union.children.len());
        assert_eq!(nested.transform.as_ref().unwrap().deref(),
                   &Transform::for_translation(4.0, 5.0, 6.0))
    }
}