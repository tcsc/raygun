use std::sync::Arc;

use nom::{
    branch::alt,
    combinator::map,
    multi::{many0, separated_list},
    IResult,
};

use raygun_math::{Point, Vector, Transform};
use raygun_material::Material;
use raygun_primitives::{
    AxisAlignedBox,
    Box as _Box, Object, ObjectList, Plane, Sphere, Union,
    //PointLight
};

use crate::{
    material::material,
    SceneRef,
    constructs::*,
    lights::point_light,
    transform::*
};

fn sphere(scene: SceneRef) 
    -> impl Fn(&[u8]) -> IResult<&[u8], Object>
{
    enum Arg {
        Radius(f64),
        Centre(Vector),
        Mat(Material),
        XForm(Transform)
    }

    move |input| {
        let rval = named_object("sphere", 
            block(separated_list(comma, alt((
                map_named_value("radius", real_number, Arg::Radius),
                map_named_value("centre", vector_literal, Arg::Centre),
                map_named_value("material", material(scene.clone()), Arg::Mat),
                map_named_value("transform", transform, Arg::XForm),
            ))))
        )(input);

        rval.map(|(i, args)| {
            let mut result = Sphere::default();
            let mut mat = Material::default();
            let mut xform = None;

            for arg in args {
                match arg {
                    Arg::Radius(r) => result.radius = r,
                    Arg::Centre(c) => result.centre = c,
                    Arg::Mat(m) => mat = m,
                    Arg::XForm(x) => xform = Some(x)
                }
            }
            
            (i, as_object(result, mat, xform))
        })
    }
}

fn _box(scene: SceneRef) 
    -> impl Fn(&[u8]) -> IResult<&[u8], Object> 
{
    enum Arg {
        Upper(Point),
        Lower(Point),
        Mat(Material),
        XForm(Transform)
    };

    move |input| {
        let parse_args = named_object("box",
            block(separated_list(comma, alt((
                map_named_value("upper", vector_literal, Arg::Upper),
                map_named_value("lower", vector_literal, Arg::Lower),
                map_named_value("material", material(scene.clone()), Arg::Mat),
                map_named_value("transform", transform, Arg::XForm) 
            )))));

        let construct_box = |args: Vec<Arg>| -> Object {
            let mut aab = AxisAlignedBox::default();
            let mut mat = Material::default();
            let mut xform = None;

            for arg in args {
                match arg {
                    Arg::Upper(r) => aab.upper = r,
                    Arg::Lower(c) => aab.lower = c,
                    Arg::Mat(m) => mat = m,
                    Arg::XForm(x) => xform = Some(x)
                }
            }
            
            as_object(_Box::from(aab), mat, xform)
        };

        map(parse_args, construct_box)(input)
    }
}

fn plane(scene: SceneRef) 
    -> impl Fn(&[u8]) -> IResult<&[u8], Object>
{
    enum Arg {
        Normal(Vector),
        Offset(f64),
        Material(Material),
        XForm(Transform)
    };

    move |input| {
        let plane_block = named_object("plane",
            block(separated_list(comma, alt((
                map_named_value("normal", vector_literal, Arg::Normal),
                map_named_value("offset", real_number, Arg::Offset),
                map_named_value("material", material(scene.clone()), Arg::Material),
                map_named_value("transfomr", transform, Arg::XForm)
            )))));
        
        let construct_plane = |args| {
            let mut p = Plane::default();
            let mut mat = Material::default();
            let mut xform = None;
    
            for arg in args {
                match arg {
                    Arg::Normal(n) => p.normal = n.normalize(),
                    Arg::Offset(o) => p.offset = o,
                    Arg::Material(m) => mat = m,
                    Arg::XForm(x) => xform = Some(x)
                }
            }

            as_object(p, mat, xform)
        };

        map(plane_block, construct_plane)(input)
    }
}

///
/// Parses a group of objects, arbitrarily transformed. Transforms are applied in the order
/// they're encountered, and nested groups are allowed.
///
fn union(scene: SceneRef) 
    -> impl Fn(&[u8]) -> IResult<&[u8], Object>
{
    enum Arg {
        XForm(Transform),
        Material(Material),
        Children(ObjectList)
    };

    move |input| {
        let children = ws(block(primitives(scene.clone())));

        let union_block = named_object("union",
            block(separated_list(comma, alt((
                map_named_value("transform", transform, Arg::XForm),
                map_named_value("material", material(scene.clone()),
                                Arg::Material),
                map_named_value("objects", children, Arg::Children)
            ))))
        );

        union_block(input)
            .map(|(i, args)| {
                let mut u = Union::default();
                let mut mat = Material::default();
                let mut xform = None;
        
                for arg in args {
                    match arg {
                        Arg::Children(c) => u.children = c,
                        Arg::Material(m) => mat = m,
                        Arg::XForm(x) => xform = Some(x)
                    }
                }
                
                (i, as_object(u, mat, xform))
            })
    }
}

fn primitive<'a>(scene: SceneRef) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], Arc<Object>> 
{
    let p = ws(alt((
        sphere(scene.clone()),
        _box(scene.clone()),
        plane(scene.clone()),
        point_light(scene.clone()),
        union(scene.clone())        
    )));
    
    map(p, Arc::new)
}

pub fn primitives<'a>(scene: SceneRef)
    -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Vec<Arc<Object>>>
{
    many0(primitive(scene))
}

#[cfg(test)]
mod test {
    use super::*;

    use std::ops::Deref;
    use float_cmp::ApproxEqUlps;
    use raygun_math::{point, vector, degrees};
    use raygun_primitives::{Box as _Box, Plane, Sphere};

    #[test]
    fn parse_sphere() {
        let state = SceneRef::default();

        let (_, obj) = sphere(state)(
            b"sphere { radius: 1.2340, centre: {1, 2, 3} }",
            ).unwrap();

        let s = obj.as_primitive::<Sphere>().unwrap();
        assert_eq!(s.radius, 1.234);
        assert_eq!(s.centre, point(1.0, 2.0, 3.0));
    }

    #[test]
    fn parse_sphere_default() {
        let state = SceneRef::default();
        let (_, obj) = sphere(state)(b"sphere { }").unwrap();

        let s = obj.as_primitive::<Sphere>().unwrap();
        assert_eq!(s.radius, 1.0);
        assert_eq!(s.centre, point(0.0, 0.0, 0.0));
    }

    #[test]
    fn parse_box() {
        let state = SceneRef::default();

        let (_, obj) = 
            _box(state)(b"box { lower: {1,2,3}, upper: {4.1, 5.2, 6.3} }").unwrap();

        let b = obj.as_primitive::<_Box>().unwrap();
        assert!(b.lower().approx_eq(point(1.0, 2.0, 3.0)),
                "Actual: {:?}", b.lower());
        assert!(b.upper().approx_eq(point(4.1, 5.2, 6.3)));
    }

    #[test]
    fn parse_plane() {
        let state = SceneRef::default();

        let (_, obj) = plane(state)(b"plane { normal: {1.2, 3.4, 5.6}, offset: 7.8 }")
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

        let state = SceneRef::default();
        let (_, obj) = union(state)(text.as_bytes()).unwrap();

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

        let state = SceneRef::default();
        let (_, obj) = union(state)(text.as_bytes()).unwrap();
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