use crate::constructs::*;
use nom::{branch::alt, combinator::map, multi::separated_list, IResult};
use raygun_material::Opacity;

pub fn parse(input: &[u8]) -> IResult<&[u8], Opacity> {
    enum Arg {
        Alpha(f64),
        RefractiveIndex(f64),
    };

    let parse_block = block(separated_list(
        comma,
        alt((
            map_named_value("alpha", real_number, Arg::Alpha),
            map_named_value("refractive_index", real_number, Arg::RefractiveIndex),
        )),
    ));

    let construct = |args| {
        let mut opacity = Opacity::default();
        for arg in args {
            match arg {
                Arg::Alpha(a) => opacity.alpha = a,
                Arg::RefractiveIndex(n) => opacity.refractive_index = n,
            }
        }
        opacity
    };

    map(parse_block, construct)(input)
}

#[cfg(test)]
mod test {
    use float_cmp::ApproxEqUlps;

    macro_rules! opapcity_tests {
        ($($name:ident: $text:expr, ($expected_alpha:expr, $expected_n:expr),)*) => {
            $(
                #[test]
                fn $name() {
                    let (_, o) = super::parse($text.as_bytes()).unwrap();
                    assert!(o.alpha.approx_eq_ulps(&$expected_alpha, 1));
                    assert!(o.refractive_index.approx_eq_ulps(&$expected_n, 1));
                }
            )*
        }
    }

    opapcity_tests! {
        fully_specified: "{alpha: 0.5, refractive_index: 0.2}", (0.5, 0.2),
        extra_spacing: "{  alpha: 0.5,  refractive_index: 0.2  }", (0.5, 0.2),
        packed: "{alpha:0.5,refractive_index:0.2}", (0.5, 0.2),
        default_alpha: "{refractive_index: 0.2}", (1.0, 0.2),
        default_refractive_index: "{alpha: 0.5}", (0.5, 1.0),
    }
}
