
use nom::{
    branch::alt,
    multi::separated_list,
    IResult,
};

use raygun_math::{Transform, Vector};
use raygun_material::Material;
use raygun_primitives::{Object, Sphere};

use crate::{
    SceneRef,
    constructs::*,
    material::*,
    transform::*
};

pub fn parse(scene: SceneRef) -> impl Fn(&[u8]) -> IResult<&[u8], Object>
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


#[cfg(test)]
mod test {
    use super::*;
    use raygun_math::point;

    #[test]
    fn parse_default() {
        let state = SceneRef::default();
        let (_, obj) = super::parse(state)(b"sphere { }").unwrap();

        let s = obj.as_primitive::<Sphere>().unwrap();
        assert_eq!(s.radius, 1.0);
        assert_eq!(s.centre, point(0.0, 0.0, 0.0));
    }

    #[test]
    fn parse() {
        let state = SceneRef::default();

        let (_, obj) = super::parse(state)(
            b"sphere { radius: 1.2340, centre: {1, 2, 3} }",
            ).unwrap();

        let s = obj.as_primitive::<Sphere>().unwrap();
        assert_eq!(s.radius, 1.234);
        assert_eq!(s.centre, point(1.0, 2.0, 3.0));
    }
}