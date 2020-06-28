use super::{
    constructs::*,
    colour::*
};

use raygun_math::Point;
use raygun_material::{Colour, Material};
use raygun_primitives::{Object, PointLight};

use nom::IResult;

pub fn point_light(_scene: SceneRef) -> 
    impl Fn(&[u8]) -> IResult<&[u8], Object>
{
    use nom::{
        branch::alt,
        multi::separated_list
    };

    enum Args {
        Col(Colour),
        Loc(Point)
    };

    move |input| {
        let p = named_object("point_light",
            block(separated_list(comma, 
                ws(alt((
                    map_named_value("colour", colour_literal, Args::Col),
                    map_named_value("location", vector_literal, Args::Loc)
                )))
            ))
        );
        
        p(input).map(|(i, args)| {
            let mut result = PointLight::default();
            for arg in args {
                match arg {
                    Args::Loc(l) => result.loc = l,
                    Args::Col(c) => result.colour = c
                }
            }
            (i, as_object(result, Material::default(), None))
        })
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use raygun_math::point;
    use raygun_material::Colour;
    use raygun_primitives::PointLight;

    #[test]
    fn parse_point_light() {
        use nom::IResult;

        let state = SceneRef::default();

        match point_light(state)(b"point_light { colour: {0.3, 0.4, 0.5}, location: {1, 2, 3} }") {
            IResult::Ok((_, obj)) => {
                let l = obj.as_primitive::<PointLight>().unwrap();
                assert_eq!(l.colour, Colour::new(0.3, 0.4, 0.5));
                assert_eq!(l.loc, point(1.0, 2.0, 3.0));
            }
            IResult::Err(_) => assert!(false),
        }
    }
}
