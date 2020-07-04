use crate::{Colour, Finish, Opacity, Pigment};
use raygun_math::Point;

#[derive(Debug, Default)]
pub struct Material {
    // transforms, etc
    pub finish: Finish,
    pub pigment: Pigment,
    pub opacity: Opacity,
}

impl Material {
    pub fn sample<'a>(&'a self, _p: Point) -> (Colour, &'a Finish) {
        match self.pigment {
            Pigment::Solid(c) => (c, &self.finish),
        }
    }
}
