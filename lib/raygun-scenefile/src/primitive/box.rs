
use nom::{
    branch::alt,
    combinator::map,
    IResult,
    multi::separated_list,
};

use crate::{
    SceneRef,
    constructs::*,
    material::*,
    transform::*
};
use raygun_math::{Transform, Point};
use raygun_material::Material;
use raygun_primitives::{Object, Box, AxisAlignedBox};

pub fn parse(scene: SceneRef) -> impl Fn(&[u8]) -> IResult<&[u8], Object> 
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
            
            as_object(Box::from(aab), mat, xform)
        };

        map(parse_args, construct_box)(input)
    }
}

#[cfg(test)]
mod test {
    use crate::SceneRef;
    use raygun_primitives::Box; 
    use raygun_math::point;

    #[test]
    fn parse_box() {
        let state = SceneRef::default();

        let (_, obj) = 
            super::parse(state)(b"box { lower: {1,2,3}, upper: {4.1, 5.2, 6.3} }").unwrap();

        let b = obj.as_primitive::<Box>().unwrap();
        assert!(b.lower().approx_eq(point(1.0, 2.0, 3.0)),
                "Actual: {:?}", b.lower());
        assert!(b.upper().approx_eq(point(4.1, 5.2, 6.3)));
    }


}