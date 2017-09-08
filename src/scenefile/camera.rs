use nom::{IResult};

use super::constructs::*;
use math::{point};
use units::{degrees, Degrees};
use camera::Camera;

// ////////////////////////////////////////////////////////////////////////////
// Camera
// ////////////////////////////////////////////////////////////////////////////

pub fn camera<'a>(input: &'a[u8], state: &SceneState) -> IResult<&'a [u8], Camera> {
    let mut loc = point(0.0, 0.0, 0.0);
    let mut target = point(0.0, 0.0, 0.0);
    let mut sky = point(0.0, 1.0, 0.0);
    let mut fov = degrees(39.0).radians();

    let rval = {
        named_object!(input, "camera",
            block!(separated_list!(comma,
                ws!(alt!(
                    call!(named_value, "location", vector_literal, |l| { loc = l; }) |
                    call!(named_value, "sky", vector_literal, |s| { sky = s; }) |
                    call!(named_value, "look_at", vector_literal, |l| { target = l; }) |
                    call!(named_value, "field_of_view", real_number, |f| { fov = degrees(f).radians(); })
                ))
        )))
    };

    match rval {
        IResult::Done(i, _) => {
            let dir = (target - loc).normalize();
            let right = sky.cross(dir).normalize();
            let up = dir.cross(right).normalize();
            let aspect_ratio = state.width as f64 / state.height as f64;
            let new_camera = Camera {
                loc: loc,
                dir: dir,
                right: right,
                up: up,
                hfov: fov,
                vfov: fov / aspect_ratio
            };

            debug!("Camera definition {:?}", new_camera);
            IResult::Done(i, new_camera)
        },
        IResult::Error(e) => IResult::Error(e),
        IResult::Incomplete(x) => IResult::Incomplete(x)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::IResult;
    use float_cmp::ApproxEqUlps;

    #[test]
    fn parse_minimal_camera() {
        use math::{point, vector};
        use std::f64::consts::FRAC_1_SQRT_2;

        let state = SceneState::default();
        let text = r#"camera {
            location: { 10.0, 10.0, -10.0 },
            look_at: {0.0, 0.0, 0.0}
        }"#;

        let expected_loc   = vector(10.0, 10.0, -10.0);
        let expected_dir   = vector(-1.0, -1.0,   1.0).normalize();
        let expected_right = vector( 1.0,  0.0,   1.0).normalize();
        let expected_up    = vector(-1.0,  2.0,   1.0).normalize();

        match camera(text.as_bytes(), &state) {
            IResult::Done(_, cam) => {
                assert!(cam.loc.approx_eq(expected_loc),
                    "Expected {:?}, actual {:?}", expected_loc, cam.loc);

                assert!(cam.dir.approx_eq(expected_dir),
                    "Expected {:?}, actual {:?}", expected_dir, cam.dir);
                assert!(cam.right.approx_eq(expected_right),
                    "Expected {:?}, actual {:?}", expected_right, cam.right);
                assert!(cam.up.approx_eq(expected_up),
                    "Expected {:?}, actual {:?}", expected_up, cam.up);

                assert_eq!(degrees(39.0).radians(), cam.hfov);

                assert_eq!(degrees(39.0 * (3.0/4.0)).radians(), cam.vfov);

            },

            IResult::Error(e) =>  assert!(false, "Parse failed: {:?}", e),
            IResult::Incomplete(_) =>  assert!(false)
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
            IResult::Done(_, cam) => {
                assert!(cam.hfov.get().approx_eq_ulps(&(PI/2.0), 2));

                let vfov = 0.75 * PI / 2.0;
                assert!(cam.vfov.get().approx_eq_ulps(&vfov, 2),
                    "Expected {:?}, actual {:?}", vfov, cam.vfov.get());
            },
            IResult::Error(e) =>  assert!(false, "Parse failed: {:?}", e),
            IResult::Incomplete(_) =>  assert!(false)
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
            IResult::Done(_, cam) => {
                let hfov = PI/2.0;
                assert!(cam.hfov.get().approx_eq_ulps(&hfov, 2),
                        "Expected {:?}, actual {:?}", hfov, cam.hfov.get());

                let vfov = 0.5625 * PI / 2.0;
                assert!(cam.vfov.get().approx_eq_ulps(&vfov, 2),
                        "Expected {:?}, actual {:?}", vfov, cam.vfov.get());
            },
            IResult::Error(e) =>  assert!(false, "Parse failed: {:?}", e),
            IResult::Incomplete(_) =>  assert!(false)
        }
    }
}