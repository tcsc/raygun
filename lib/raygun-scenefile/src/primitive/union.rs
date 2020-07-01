use nom::{branch::alt, multi::separated_list, IResult};

use raygun_material::Material;
use raygun_math::Transform;
use raygun_primitives::{Object, ObjectList, Union};

use super::primitives;
use crate::{constructs::*, material::*, transform::*, SceneRef};

///
/// Parses a group of objects, arbitrarily transformed. Transforms are applied in the order
/// they're encountered, and nested groups are allowed.
///
pub fn parse(scene: SceneRef) -> impl Fn(&[u8]) -> IResult<&[u8], Object> {
    enum Arg {
        XForm(Transform),
        Material(Material),
        Children(ObjectList),
    };

    move |input| {
        let children = ws(block(primitives(scene.clone())));

        let union_block = named_object(
            "union",
            block(separated_list(
                comma,
                alt((
                    map_named_value("transform", transform, Arg::XForm),
                    map_named_value("material", material(scene.clone()), Arg::Material),
                    map_named_value("objects", children, Arg::Children),
                )),
            )),
        );

        union_block(input).map(|(i, args)| {
            let mut u = Union::default();
            let mut mat = Material::default();
            let mut xform = None;

            for arg in args {
                match arg {
                    Arg::Children(c) => u.children = c,
                    Arg::Material(m) => mat = m,
                    Arg::XForm(x) => xform = Some(x),
                }
            }

            (i, as_object(u, mat, xform))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use raygun_math::degrees;
    use std::ops::Deref;

    #[test]
    pub fn parse() {
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
        let (_, obj) = super::parse(state)(text.as_bytes()).unwrap();

        let expected_transform = Transform::for_translation(1.0, 2.0, 3.0)
            .rotate(
                degrees(90.0).radians(),
                degrees(45.0).radians(),
                degrees(180.0).radians(),
            )
            .scale(0.5, 2.0, 1.5);
        assert_eq!(obj.transform.as_ref().unwrap().deref(), &expected_transform);

        let u = obj.as_primitive::<Union>().unwrap();
        assert_eq!(u.children.len(), 3);
    }

    #[test]
    fn parse_nested_union() {
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
        let (_, obj) = super::parse(state)(text.as_bytes()).unwrap();
        let base = Transform::for_translation(1.0, 2.0, 3.0);

        let u = obj.as_primitive::<Union>().unwrap();

        assert_eq!(3, u.children.len());
        assert_eq!(base, *obj.transform.as_ref().unwrap().deref());

        let nested = &u.children[1];
        let nested_union = nested.as_primitive::<Union>().unwrap();
        assert_eq!(1, nested_union.children.len());
        assert_eq!(
            nested.transform.as_ref().unwrap().deref(),
            &Transform::for_translation(4.0, 5.0, 6.0)
        )
    }
}
