use raygun_math::*;

use log::debug;

#[derive(Debug)]
pub struct Camera {
    pub loc: Point,
    pub dir: Vector,
    pub up: Vector,
    pub right: Vector,
    pub hfov: Angle<Radians>,
    pub vfov: Angle<Radians>,
}

impl Default for Camera {
    ///
    /// Roughly simulates the field of view of a 50mm lens on a 35mm
    /// camera.
    ///
    fn default() -> Camera {
        Camera {
            loc: point(0.0, 0.0, 0.0),
            dir: vector(0.0, 0.0, 1.0),
            up: vector(0.0, 1.0, 0.0),
            right: vector(1.0, 0.0, 0.0),
            hfov: degrees(39.0).radians(),
            vfov: degrees(27.0).radians(),
        }
    }
}

impl Camera {
    pub fn with_loc(&self, x: f64, y: f64, z: f64) -> Camera {
        Camera {
            loc: point(x, y, z),
            ..*self
        }
    }

    pub fn with_dir(&self, x: f64, y: f64, z: f64) -> Camera {
        Camera {
            dir: vector(x, y, z),
            ..*self
        }
    }

    pub fn projector(&self, width: isize, height: isize) -> Projection {
        //                              tan(field-of-view/2)
        // up                  -------------
        // |                    \    |    /
        // |                     \   |   /
        // * --> dir              \  |1 /
        //  \                      \ | /
        //   \                      \|/
        //     right
        //
        debug!("Generating projection...");

        let plane_centre = self.loc + self.dir;

        let half_hfov: Angle<Radians> = self.hfov / 2.0;
        let width_v = self.right * half_hfov.tan();
        let centre_left = plane_centre - width_v;
        let dx = (width_v * 2) / width;

        let half_vfov: Angle<Radians> = self.vfov / 2.0;
        let height_v = self.up * half_vfov.tan();
        let top_left = centre_left + height_v;
        let dy = (-height_v * 2) / height;

        debug!("Plane Centre: {:?}", plane_centre);
        debug!("Plane Left:   {:?}", centre_left);
        debug!("Top-left:     {:?}", top_left);
        debug!("dx:           {:?}", dx);
        debug!("dy:           {:?}", dy);

        Projection {
            topleft: top_left,
            dx: dx,
            dy: dy,
            src: self.loc,
        }
    }
}

pub struct Projection {
    topleft: Point,
    dx: Vector,
    dy: Vector,
    src: Point,
}

impl Projection {
    pub fn ray_for(&self, x: u32, y: u32) -> Ray {
        let pixel_pos = self.topleft + (x * self.dx) + (y * self.dy);
        let v = Vector::between(self.src, pixel_pos).normalize();
        Ray::new(self.src, v)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn projection_quadrants_look_good() {
        let c = Camera::default().with_loc(0.0, 0.0, -1.0);
        let p = c.projector(640, 480);

        let topleft = p.ray_for(0, 0);
        assert_eq!(topleft.src, c.loc);
        assert!(topleft.dir.x < 0.0);
        assert!(topleft.dir.y > 0.0);
        assert!(topleft.dir.z < 1.0);

        let topright = p.ray_for(639, 0);
        assert_eq!(topright.src, c.loc);
        assert!(topright.dir.x > 0.0);
        assert!(topright.dir.y > 0.0);
        assert!(topright.dir.z < 1.0);

        let bottomleft = p.ray_for(0, 479);
        assert_eq!(bottomleft.src, c.loc);
        assert!(bottomleft.dir.x < 0.0);
        assert!(bottomleft.dir.y < 0.0);
        assert!(bottomleft.dir.z < 1.0);

        let bottomright = p.ray_for(639, 479);
        assert_eq!(bottomright.src, c.loc);
        assert!(bottomright.dir.x > 0.0);
        assert!(bottomright.dir.y < 0.0);
        assert!(bottomright.dir.z < 1.0);
    }
}
