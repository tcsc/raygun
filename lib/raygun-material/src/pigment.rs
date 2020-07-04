use crate::Colour;

#[derive(Debug)]
pub enum Pigment {
    Solid(Colour),
}

impl Default for Pigment {
    fn default() -> Pigment {
        Pigment::Solid(Colour::default())
    }
}
