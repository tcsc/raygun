use crate::Colour;
use raygun_math::Point;

#[derive(Debug)]
pub struct Finish {
    pub opacity: f64,
    pub reflection: f64,
    pub ambient: f64,
    pub diffuse: f64,
    pub highlight_hardness: f64,
}

impl Default for Finish {
    fn default() -> Finish {
        Finish {
            opacity: 1.0,
            reflection: 0.0,
            ambient: 0.1,
            diffuse: 0.75,
            highlight_hardness: 500.0,
        }
    }
}

#[derive(Debug)]
pub enum Pigment {
    Solid(Colour),
}

impl Default for Pigment {
    fn default() -> Pigment {
        Pigment::Solid(Colour::default())
    }
}

#[derive(Debug, Default)]
pub struct Material {
    // transforms, etc
    pub finish: Finish,
    pub pigment: Pigment,
}

impl Material {
    pub fn sample<'a>(&'a self, _p: Point) -> (Colour, &'a Finish) {
        match self.pigment {
            Pigment::Solid(c) => (c, &self.finish),
        }
    }
}
