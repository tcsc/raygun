use math::Point;
use colour::Colour;

/// What makes a light a light?
pub trait Light {
    /**
     * Does this light cast onto the given point? If so, what colour should it be?
     */
    fn lights(&self, p: Point) -> Option<Colour>;
    fn src(&self) -> Point;
}

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

    fn lights(&self, p: Point) -> Option<Colour> {
        Some(self.colour)
    }
}