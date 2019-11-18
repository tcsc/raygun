mod pigment;

use nom::{
    multi::separated_list,
    branch::alt,
    IResult
};

use super::constructs::*;
use self::pigment::pigment;
use crate::material::{Finish, Material};

pub fn finish<'a>(input: &'a [u8]) -> IResult<&'a [u8], Finish> {
    let mut result = Finish::default();

    let rval = block(
        separated_list(comma, ws(alt((
            named_value("opacity", real_number, |o| result.opacity = o),
            named_value("reflection", real_number, |r| result.reflection = r),
            named_value("ambient", real_number, |a| result.ambient = a),
            named_value("diffuse", real_number, |d| result.diffuse = d),
            named_value("highlight", real_number, |h| result.highlight_hardness = h)
        ))))
    )(input);
    rval.map(|(i, _)| (i, result))
}

pub fn material<'a>(scene: SceneRef) -> 
    impl Fn(&'a [u8]) -> IResult<&'a [u8], Material> 
{
    move |input| {
        let mut result = Material::default();

        let rval = block(
            separated_list(comma, ws(alt((
                named_value("pigment", pigment(scene), |p| result.pigment = p),
                named_value("finish", finish, |f| result.finish = f)
            ))))
        )(input);
        rval.map(|(i, _)| (i, result))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_cmp::ApproxEqUlps;
    use crate::material::Finish;

    #[test]
    fn parses_completely_specified_finish() {
        let text = r#"{
            opacity: 0.1,
            reflection: 0.2,
            ambient: 0.3,
            diffuse: 0.4,
            highlight: 0.5
        }"#;

        match finish(text.as_bytes()) {
            IResult::Ok((_, actual)) => {
                assert!(actual.opacity.approx_eq_ulps(&0.1, 5),
                        "Expected opacity = {}, got {}",
                        0.1,
                        actual.opacity);

                assert!(actual.reflection.approx_eq_ulps(&0.2, 5),
                        "Expected reflection = {}, got {}",
                        0.2,
                        actual.reflection);

                assert!(actual.ambient.approx_eq_ulps(&0.3, 5),
                        "Expected ambient = {}, got {}",
                        0.3,
                        actual.ambient);

                assert!(actual.diffuse.approx_eq_ulps(&0.4, 5),
                        "Expected ambient = {}, got {}",
                        0.4,
                        actual.diffuse);

                assert!(actual.highlight_hardness.approx_eq_ulps(&0.5, 5),
                        "Expected ambient = {}, got {}",
                        0.5,
                        actual.highlight_hardness);
            },

            IResult::Err(e) => assert!(false, "Parse failed: {:?}", e)
        }
    }
}
