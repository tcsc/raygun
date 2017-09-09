use super::Light;
use math::{Point, Vector};
use colour::Colour;
use ray::Ray;
use primitive::Primitive;

#[derive(Debug)]
pub struct PointLight {
    pub loc: Point,
    pub colour: Colour
}

impl PointLight {
    pub fn new(pos: Point, colour: Colour) -> PointLight {
        PointLight { loc: pos, colour: colour }
    }

    pub fn position(&self) -> Point {
        self.loc
    }

    pub fn colour(&self) -> Colour {
        self.colour
    }
}

impl Primitive for PointLight {
    fn intersects(&self, r: Ray) -> Option<f64> {
        None
    }

    fn normal(&self, pt: Point) -> Vector {
        panic!("This should never be called")
    }

    fn as_light(&self) -> Option<&Light> {
        Some(self as &Light)
    }
}

impl Default for PointLight {
    fn default() -> PointLight {
        PointLight {
            loc: Point::new(0.0, 0.0, 0.0),
            colour: Colour::default(),
        }
    }
}

impl Light for PointLight {
    fn src(&self) -> Point {
        self.loc
    }

    fn illuminates(&self, p: Point) -> Option<Colour> {
        Some(self.colour)
    }
}