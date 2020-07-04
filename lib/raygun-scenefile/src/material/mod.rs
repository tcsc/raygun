mod opacity;
mod pigment;

use nom::{branch::alt, multi::separated_list, IResult};

use self::pigment::pigment;
use super::constructs::*;
use raygun_material::{Finish, Material, Opacity, Pigment};

pub fn finish<'a>(_scene: SceneRef) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Finish> {
    enum Arg {
        Reflection(f64),
        Ambient(f64),
        Diffuse(f64),
        Highlight(f64),
    };

    move |input| {
        let material_block = block(separated_list(
            comma,
            ws(alt((
                map_named_value("reflection", real_number, Arg::Reflection),
                map_named_value("ambient", real_number, Arg::Ambient),
                map_named_value("diffuse", real_number, Arg::Diffuse),
                map_named_value("highlight", real_number, Arg::Highlight),
            ))),
        ));

        material_block(input).map(|(i, args)| {
            let mut result = Finish::default();

            for arg in args {
                match arg {
                    Arg::Reflection(r) => result.reflection = r,
                    Arg::Ambient(a) => result.ambient = a,
                    Arg::Diffuse(d) => result.diffuse = d,
                    Arg::Highlight(h) => result.highlight_hardness = h,
                }
            }

            (i, result)
        })
    }
}

pub fn material<'a>(scene: SceneRef) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Material> {
    enum Arg {
        Pigment(Pigment),
        Finish(Finish),
        Opacity(Opacity),
    };

    move |input| {
        let material_block = block(separated_list(
            comma,
            ws(alt((
                map_named_value("pigment", pigment(scene.clone()), Arg::Pigment),
                map_named_value("finish", finish(scene.clone()), Arg::Finish),
                map_named_value("opacity", opacity::parse, Arg::Opacity),
            ))),
        ));

        material_block(input).map(|(i, args)| {
            let mut result = Material::default();

            for arg in args {
                match arg {
                    Arg::Finish(f) => result.finish = f,
                    Arg::Pigment(p) => result.pigment = p,
                    Arg::Opacity(o) => result.opacity = o,
                }
            }

            (i, result)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_cmp::ApproxEqUlps;
    use raygun_material::Colour;

    #[test]
    fn parses_simple_material() {
        let text = r#"{
            pigment: solid { colour: {0.1, 0.2, 0.3} },
            finish: { reflection: 0.4 },
            opacity: { alpha: 0.25, refractive_index: 1.1 }
        }"#;

        let scene = SceneRef::default();

        let (_, mat) = material(scene)(text.as_bytes()).unwrap();
        {
            let Pigment::Solid(c) = mat.pigment;
            assert!(c.approx_eq(Colour::new(0.1, 0.2, 0.3)));
        }

        assert!(mat.finish.reflection.approx_eq_ulps(&0.4, 5));
        assert!(mat.opacity.alpha.approx_eq_ulps(&0.25, 5));
        assert!(mat.opacity.refractive_index.approx_eq_ulps(&1.1, 5));
    }

    #[test]
    fn parses_completely_specified_finish() {
        let text = r#"{
            reflection: 0.2,
            ambient: 0.3,
            diffuse: 0.4,
            highlight: 0.5
        }"#;

        let scene = SceneRef::default();

        match finish(scene)(text.as_bytes()) {
            IResult::Ok((_, actual)) => {
                assert!(
                    actual.reflection.approx_eq_ulps(&0.2, 5),
                    "Expected reflection = {}, got {}",
                    0.2,
                    actual.reflection
                );

                assert!(
                    actual.ambient.approx_eq_ulps(&0.3, 5),
                    "Expected ambient = {}, got {}",
                    0.3,
                    actual.ambient
                );

                assert!(
                    actual.diffuse.approx_eq_ulps(&0.4, 5),
                    "Expected ambient = {}, got {}",
                    0.4,
                    actual.diffuse
                );

                assert!(
                    actual.highlight_hardness.approx_eq_ulps(&0.5, 5),
                    "Expected ambient = {}, got {}",
                    0.5,
                    actual.highlight_hardness
                );
            }

            IResult::Err(e) => assert!(false, "Parse failed: {:?}", e),
        }
    }
}
