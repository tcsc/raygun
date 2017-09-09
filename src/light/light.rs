use math::Point;
use colour::Colour;

/// What makes a light a light?
pub trait Light {
    /**
     * Does this light cast onto the given point? If so, what colour should it be?
     */
    fn illuminates(&self, p: Point) -> Option<Colour>;

    /**
     * What is the origin of this light?
     */
    fn src(&self) -> Point;
}