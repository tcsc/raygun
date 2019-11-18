use nom::{
    branch::alt,
    error::ParseError,
    lib::std::ops::RangeFrom,
    multi::separated_list,
    AsChar, 
    InputIter, 
    Slice,
    IResult,
};

use crate::{
    material::Material,
    math::{self, Vector, Transform},
    primitive::{AxisAlignedBox, Box as _Box, Object, Plane, Sphere, Union},
    units::degrees,
};

use std::{
    cell::RefCell,
    sync::Arc
};

use super::{
    material::material,
    SceneRef,
    constructs::*,
    lights::point_light,
    transform::*
};

fn sphere(scene: SceneRef) 
    -> impl Fn(&[u8]) -> IResult<&[u8], Object>
{
    move |input| {
        let mut result = Sphere::default();
        let mut mat = Material::default();
        let mut xform = None;

        let rval = named_object("sphere", 
            block(separated_list(comma, alt((
                named_value("radius", real_number, |r| result.radius = r),
                named_value("centre", vector_literal, |c| result.centre = c),
                named_value("material", material(scene), |m| mat = m),
                named_value("transform", transform, |t| xform = Some(t)),
            ))))
        )(input);

        rval.map(|(i, _)| {
            (i, as_object(result, mat, xform))
        })
    }
}

fn _box(scene: SceneRef) 
    -> impl Fn(&[u8]) -> IResult<&[u8], Object> 
{
    move |input| {
        let mut b = AxisAlignedBox::default();
        let mut mat = Material::default();
        let mut xform = None;

        let rval = named_object("box",
            block(separated_list(comma, alt((
                named_value("upper", vector_literal, |u| b.upper = u),
                named_value("lower", vector_literal, |l| b.lower = l),
                named_value("material", material(scene), |m| mat = m),
                named_value("transform", transform, |t| xform = Some(t)) 
            ))))
        )(input);
        rval.map(|(i,_)| (i, as_object(_Box::from(b), mat, xform)))
    }
}

fn plane(scene: SceneRef) 
    -> impl Fn(&[u8]) -> IResult<&[u8], Object>
{
    move |input| {
        let mut p = Plane::default();
        let mut mat = Material::default();
        let mut xform = None;

        let rval = named_object("plane",
            block(separated_list(comma, alt((
                named_value("normal", vector_literal, |n| p.normal = n.normalize()),
                named_value("offset", real_number, |o| p.offset = o),
                named_value("material", material(scene), |m| mat = m),
                named_value("transfomr", transform, |t| xform = Some(t))
            ))))
        )(input);
        rval.map(|(i,_)| (i, as_object(p, mat, xform)))
    }
}

///
/// Parses a group of objects, arbitrarily transformed. Transforms are applied in the order
/// they're encountered, and nested groups are allowed.
///
fn union(scene: SceneRef) 
    -> impl Fn(&[u8]) -> IResult<&[u8], Object>
{
    move |input| {
        let mut u = Union::default();
        let mut mat = Material::default();
        let mut xform = None;

        let children = ws(block(primitives(scene)));

        let rval = named_object("union",
            block(separated_list(comma, alt((
                named_value("transform", transform, |t| xform = Some(t)),
                named_value("material", material(scene), |m| mat = m),
                named_value("objects", children, |ch| u.children = ch)
            ))))
        )(input);
        rval.map(|(i,_)| (i, as_object(u, mat, xform)))    
    }
}

fn primitive<'a>(scene: SceneRef) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], Arc<Object>> 
{
    use nom::combinator::map;

    let p = ws(alt((
        sphere(scene),
        _box(scene),
        plane(scene),
        point_light(scene),
        union(scene)        
    )));
    
    map(p, Arc::new)
}

pub fn primitives<'a>(scene: SceneRef)
    -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Vec<Arc<Object>>>
{
    use nom::{
        combinator::map,
        multi::many0,
    };

    many0(primitive(scene))
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
        use crate::primitive::Sphere;
        use nom::IResult;

        let state = RefCell::new(SceneState::default());
        let (_, obj) = sphere(&state)(
            b"sphere { radius: 1.2340, centre: {1, 2, 3} }",
            ).unwrap();

        let s = obj.as_primitive::<Sphere>().unwrap();
        assert_eq!(s.radius, 1.234);
        assert_eq!(s.centre, point(1.0, 2.0, 3.0));
    }

    #[test]
    fn parse_sphere_default() {
        use crate::{
            math::point,
            primitive::Sphere
        };
        use nom::IResult;

        let state = SceneState::default();
        let (_, obj) = sphere(b"sphere { }", &state).unwrap();

        let s = obj.as_primitive::<Sphere>().unwrap();
        assert_eq!(s.radius, 1.0);
        assert_eq!(s.centre, point(0.0, 0.0, 0.0));
    }


    #[test]
    fn parse_box() {
        use crate::{
            math::point,
            primitive::Box as _Box
        };
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
        use crate::{
            math::point,
            primitive::Plane
        };
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