
use log::{debug};
use super::constructs::*;
use std::sync::Arc;

use raygun_camera::Camera;
use raygun_math::{point, Point, Vector, degrees};

use nom::{
    error::ParseError,
    lib::std::ops::RangeFrom,
    AsChar,
    InputIter,
    Slice,
    IResult,
};

// ////////////////////////////////////////////////////////////////////////////
// Camera
// ////////////////////////////////////////////////////////////////////////////

pub fn camera(state: SceneRef) -> 
    impl Fn(&[u8]) -> IResult<&[u8], Camera> 
{
    use nom::{
        branch::alt,
        multi::separated_list
    };

    enum Arg {
        Loc(Point),
        Sky(Vector),
        LookAt(Point),
        Fov(f64)
    }

    move |input| {
        let camera_block = block(separated_list(comma,
            ws(alt((
                map_named_value("location", vector_literal, Arg::Loc),
                map_named_value("sky", vector_literal, Arg::Sky),
                map_named_value("look_at", vector_literal, Arg::LookAt),
                map_named_value("field_of_view", real_number, Arg::Fov)        
            )))
        ));

        named_object("camera", camera_block)(input)
            .map(|(i, args)| {
                let mut loc = point(0.0, 0.0, 0.0);
                let mut target = point(0.0, 0.0, 0.0);
                let mut sky = point(0.0, 1.0, 0.0);
                let mut fov = degrees(39.0).radians();
        
                for arg in args {
                    match arg {
                        Arg::Loc(p) => loc = p,
                        Arg::Sky(s) => sky = s,
                        Arg::LookAt(p) => target = p,
                        Arg::Fov(d) => fov = degrees(d).radians()
                    }
                }

                (i, loc, target, sky, fov)
            })
            .map(|(i, loc, target, sky, fov)| {
                let dir = (target - loc).normalize();
                let right = sky.cross(dir).normalize();
                let up = dir.cross(right).normalize();

                let s = state.borrow();
                let aspect_ratio = s.width as f64 / s.height as f64;
                let new_camera = Camera {
                    loc: loc,
                    dir: dir,
                    right: right,
                    up: up,
                    hfov: fov,
                    vfov: fov / aspect_ratio,
                };

                debug!("Camera definition {:?}", new_camera);
                (i, new_camera)
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::IResult;
    use float_cmp::ApproxEqUlps;

    #[test]
    fn parse_minimal_camera() {
        use crate::math::{point, vector};
        use std::f64::consts::FRAC_1_SQRT_2;

        let state = SceneState::default();
        let text = r#"camera {
            location: { 10.0, 10.0, -10.0 },
            look_at: {0.0, 0.0, 0.0}
        }"#;

        let expected_loc = vector(10.0, 10.0, -10.0);
        let expected_dir = vector(-1.0, -1.0, 1.0).normalize();
        let expected_right = vector(1.0, 0.0, 1.0).normalize();
        let expected_up = vector(-1.0, 2.0, 1.0).normalize();

        match camera(text.as_bytes(), &state) {
            IResult::Ok((_, cam)) => {
                assert!(cam.loc.approx_eq(expected_loc),
                        "Expected {:?}, actual {:?}",
                        expected_loc,
                        cam.loc);

                assert!(cam.dir.approx_eq(expected_dir),
                        "Expected {:?}, actual {:?}",
                        expected_dir,
                        cam.dir);
                assert!(cam.right.approx_eq(expected_right),
                        "Expected {:?}, actual {:?}",
                        expected_right,
                        cam.right);
                assert!(cam.up.approx_eq(expected_up),
                        "Expected {:?}, actual {:?}",
                        expected_up,
                        cam.up);

                assert_eq!(degrees(39.0).radians(), cam.hfov);

                assert_eq!(degrees(39.0 * (3.0 / 4.0)).radians(), cam.vfov);
            },            
            IResult::Err(e) => assert!(false, "Parse failed: {:?}", e)
        }
    }

    #[test]
    fn honours_fov() {
        use std::f64::consts::PI;
        let state = SceneState::default();

        let text = r#"camera {
            field_of_view: 90
        }"#;

        match camera(text.as_bytes(), &state) {
            IResult::Ok((_, cam)) => {
                assert!(cam.hfov.get().approx_eq_ulps(&(PI / 2.0), 2));

                let vfov = 0.75 * PI / 2.0;
                assert!(cam.vfov.get().approx_eq_ulps(&vfov, 2),
                        "Expected {:?}, actual {:?}",
                        vfov,
                        cam.vfov.get());
            }
            IResult::Err(e) => assert!(false, "Parse failed: {:?}", e)
        }
    }

    #[test]
    fn honours_aspect_ratio() {
        use std::f64::consts::PI;
        let state = SceneState::new(1920, 1080);

        let text = r#"camera {
            field_of_view: 90
        }"#;

        match camera(text.as_bytes(), &state) {
            IResult::Ok((_, cam)) => {
                let hfov = PI / 2.0;
                assert!(cam.hfov.get().approx_eq_ulps(&hfov, 2),
                        "Expected {:?}, actual {:?}",
                        hfov,
                        cam.hfov.get());

                let vfov = 0.5625 * PI / 2.0;
                assert!(cam.vfov.get().approx_eq_ulps(&vfov, 2),
                        "Expected {:?}, actual {:?}",
                        vfov,
                        cam.vfov.get());
            }
            IResult::Err(e) => assert!(false, "Parse failed: {:?}", e)
        }
    }
}
