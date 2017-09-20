mod pigment;

use nom::IResult;

use super::constructs::*;
use self::pigment::pigment;
use material::{Finish, Material};

pub fn finish<'a>(input: &'a [u8]) -> IResult<&'a [u8], Finish> {
    let mut result = Finish::default();

    let rval = {
        block!(input,
               separated_list!(comma,
                               ws!(alt!(call!(named_value,
                                              "opacity",
                                              real_number,
                                              set!(result.opacity)) |
                                        call!(named_value,
                                              "reflection",
                                              real_number,
                                              set!(result.reflection)) |
                                        call!(named_value,
                                              "ambient",
                                              real_number,
                                              set!(result.ambient)) |
                                        call!(named_value,
                                              "diffuse",
                                              real_number,
                                              set!(result.diffuse)) |
                                        call!(named_value,
                                              "highlight",
                                              real_number,
                                              set!(result.highlight_hardness))))))
    };

    match rval {
        IResult::Done(i, _) => IResult::Done(i, result),
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x),
    }
}

pub fn material<'a>(input: &'a [u8]) -> IResult<&'a [u8], Material> {
    let mut result = Material::default();

    let rval = {
        block!(input,
               separated_list!(comma,
                               ws!(alt!(call!(named_value,
                                              "pigment",
                                              pigment,
                                              set!(result.pigment)) |
                                        call!(named_value,
                                              "finish",
                                              finish,
                                              set!(result.finish))))))
    };

    match rval {
        IResult::Done(i, _) => IResult::Done(i, result),
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use float_cmp::ApproxEqUlps;

    #[test]
    fn parses_completely_specified_finish() {
        use material::Finish;

        let text = r#"{
            opacity: 0.1,
            reflection: 0.2,
            ambient: 0.3,
            diffuse: 0.4,
            highlight: 0.5
        }"#;

        match finish(text.as_bytes()) {
            IResult::Done(_, actual) => {
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
            }
            IResult::Error(e) => assert!(false, "Parse failed: {:?}", e),
            IResult::Incomplete(_) => assert!(false),
        }
    }
}
