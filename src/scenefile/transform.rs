use nom::IResult;
use math::{self, Vector, Transform, translation_matrix};
use units::degrees;
use super::constructs::*;


fn translate<'a>(input: &'a [u8]) -> IResult<&'a [u8], Transform>  {
    named_value(input, "translate", vector_literal,
                | Vector {x, y, z} | {
                    Transform::identity().translate(x, y, z)
                })
}

fn rotate<'a>(input: &'a [u8]) -> IResult<&'a [u8], Transform>  {
    named_value(input, "rotate", vector_literal,
                | Vector {x, y, z} | {
                    Transform::identity().rotate(degrees(x).radians(),
                                                 degrees(y).radians(),
                                                 degrees(z).radians())
                })
}

fn scale<'a>(input: &'a [u8]) -> IResult<&'a [u8], Transform> {
    named_value(input, "scale", vector_literal,
                | Vector { x, y, z } | {
                    Transform::identity().scale(x, y, z)
                })
}

pub fn transform<'a>(input: &'a [u8]) -> IResult<&'a [u8], Transform> {
    let mut result = Transform::identity();
    let rval = {
        block!(input, separated_list!(comma,
            ws!(map!(
                alt!(translate | rotate | scale),
                |t| { result = result.apply(&t) }
            ))
        ))
    };

    rval.map(|_| result)
}

#[cfg(test)]
mod test {
    use super::*;
    use nom;
    use units::degrees;

    #[test]
    fn parse_translate() {
        let text = "translate: { 1.2, 3, -4 }";

        let (_, t) = translate(text.as_bytes()).unwrap();
        let expected = Transform::identity().translate(1.2, 3.0, -4.0);
        assert_eq!(t, expected,
            "Expected: {:?}\nActual {:?}", expected, t);
    }

    #[test]
    fn parse_rotate() {
        let text = "rotate: { 5, -6.7, 8 }";

        let (_, t) = rotate(text.as_bytes()).unwrap();
        let expected = Transform::identity().rotate(
            degrees(5.0).radians(),
            degrees(-6.7).radians(),
            degrees(8.0).radians());
        assert_eq!(t, expected,
                   "Expected: {:?}\nActual {:?}", expected, t);
    }

    #[test]
    fn parse_scale() {
        let text = "scale: { 3, 2, 1 }";

        let (_, t) = scale(text.as_bytes()).unwrap();
        let expected = Transform::identity().scale(3.0, 2.0, 1.0);
        assert_eq!(t, expected,
                   "Expected: {:?}\nActual {:?}", expected, t);
    }

    #[test]
    fn parse_transform_block() {
        let text = r#"{
            translate: {1, 2, 3},
            rotate: {45, 90, 135},
            scale: {0.5, 1.0, 1.5},
            translate: {-1, -2, -3},
            translate: {6, 7, 8}
        }"#;

        let (_, t) = transform(text.as_bytes())
            .map_err(|e| { println!("error: {}", e.description()); e })
            .unwrap();
        let expected = Transform::identity()
            .translate(1.0, 2.0, 3.0)
            .rotate(degrees(45.0).radians(),
                    degrees(90.0).radians(),
                    degrees(135.0).radians())
            .scale(0.5, 1.0, 1.5)
            .translate(-1.0, -2.0, -3.0)
            .translate(6.0, 7.0, 8.0);

        assert_eq!(t, expected);
    }
}