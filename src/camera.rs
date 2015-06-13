use math::*;
use ray::Ray;
use units::{Angle, Radians, degrees};

pub struct Camera {
    loc:  Point,
    dir:  Vector,
    up:   Vector,
    hfov: Angle<Radians>,
    vfov: Angle<Radians>
}

impl Default for Camera {
    ///
    /// Roughly simulates the field of view of a 50mm lens on a 35mm
    /// camera.
    ///
    fn default() -> Camera {
        Camera {
            loc: point( 0.0, 0.0, 0.0),
            dir: vector(0.0, 0.0, 1.0),
            up:  vector(0.0, 1.0, 0.0),
            hfov: degrees(39.6).radians(),
            vfov: degrees(27.0).radians()
        }
    }
}

impl Camera {
    pub fn with_loc(&self, x: f64, y: f64, z: f64) -> Camera {
        Camera{ loc: point(x, y, z), .. *self }
    }

    pub fn loc(&self) -> Point {
        self.loc
    }

    pub fn dir(&self, x: f64, y: f64, z: f64) -> Camera {
        Camera{ dir: vector(x, y, z), .. *self }
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
        let right = self.up.cross(self.dir).normalize();
        let vy = -self.up;
        let plane_centre = self.loc + self.dir;

        let width_v = right * (self.hfov / 2isize).tan();
        let centre_left = plane_centre - width_v;
        let dx = (width_v * 2) / width;

        let height_v = self.up * (self.vfov / 2isize).tan();
        let top_left = centre_left + height_v;
        let dy = (-height_v * 2) / height;

        Projection {
            topleft: top_left,
            dx: dx,
            dy: dy,
            width: width,
            height: height,
            src: self.loc,
        }
    }
}

struct Projection {
    topleft: Point,
    dx: Vector,
    dy: Vector,
    width: isize,
    height: isize,
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

        let topleft = p.ray_for(0,0);
        assert_eq!(topleft.src, c.loc());
        assert!(topleft.dir.x < 0.0);
        assert!(topleft.dir.y > 0.0);
        assert!(topleft.dir.z < 1.0);

        let topright = p.ray_for(639,0);
        assert_eq!(topright.src, c.loc());
        assert!(topright.dir.x > 0.0);
        assert!(topright.dir.y > 0.0);
        assert!(topright.dir.z < 1.0);

        let bottomleft = p.ray_for(0,479);
        assert_eq!(bottomleft.src, c.loc());
        assert!(bottomleft.dir.x < 0.0);
        assert!(bottomleft.dir.y < 0.0);
        assert!(bottomleft.dir.z < 1.0);

        let bottomright = p.ray_for(639, 479);
        assert_eq!(bottomright.src, c.loc());
        assert!(bottomright.dir.x > 0.0);
        assert!(bottomright.dir.y < 0.0);
        assert!(bottomright.dir.z < 1.0);
    }
}
