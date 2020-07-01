use nom::{branch::alt, combinator::map, multi::separated_list, IResult};

use raygun_material::Material;
use raygun_math::{Transform, Vector};
use raygun_primitives::{Object, Plane};

use crate::{constructs::*, material::*, transform::*, SceneRef};

pub fn parse(scene: SceneRef) -> impl Fn(&[u8]) -> IResult<&[u8], Object> {
    enum Arg {
        Normal(Vector),
        Offset(f64),
        Material(Material),
        XForm(Transform),
    };

    move |input| {
        let plane_block = named_object(
            "plane",
            block(separated_list(
                comma,
                alt((
                    map_named_value("normal", vector_literal, Arg::Normal),
                    map_named_value("offset", real_number, Arg::Offset),
                    map_named_value("material", material(scene.clone()), Arg::Material),
                    map_named_value("transfomr", transform, Arg::XForm),
                )),
            )),
        );

        let construct_plane = |args| {
            let mut p = Plane::default();
            let mut mat = Material::default();
            let mut xform = None;

            for arg in args {
                match arg {
                    Arg::Normal(n) => p.normal = n.normalize(),
                    Arg::Offset(o) => p.offset = o,
                    Arg::Material(m) => mat = m,
                    Arg::XForm(x) => xform = Some(x),
                }
            }

            as_object(p, mat, xform)
        };

        map(plane_block, construct_plane)(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_cmp::ApproxEqUlps;
    use raygun_math::vector;

    #[test]
    fn parse_plane() {
        let state = SceneRef::default();

        let (_, obj) =
            super::parse(state)(b"plane { normal: {1.2, 3.4, 5.6}, offset: 7.8 }").unwrap();

        let p = obj.as_primitive::<Plane>().unwrap();
        let expected = vector(0.1801712440614613, 0.5104851915074736, 0.8407991389534859);

        assert!(
            p.normal.approx_eq(expected),
            "Expected normal {:?}, got {:?}",
            expected,
            p.normal
        );
        assert!(p.offset.approx_eq_ulps(&7.8, 1));
    }
}
