use math::Point;
use colour::Colour;

pub struct PointLight {
	loc: Point,
	colour: Colour
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