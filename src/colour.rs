use std::ops;

///
/// A colour value, with each channel normalised between 0 and 1
///
#[derive(Copy, Clone, Debug)]
pub struct Colour { pub r: f64, pub g: f64, pub b: f64 }

pub const black : Colour = Colour { r: 0.0, g: 0.0, b: 0.0 };
pub const white : Colour = Colour { r: 1.0, g: 1.0, b: 1.0 };

impl Colour {
    ///
    /// Is this colour the same as another colour, or at least close enough for
    /// practical purposes.
    ///
    fn approx_eq(&self, other: Colour) -> bool {
        (self.r - other.r).abs() < 1e-10 &&
        (self.g - other.g).abs() < 1e-10 &&
        (self.b - other.b).abs() < 1e-10
    }
}

impl ops::Add<Colour> for Colour {
    type Output = Colour;

    #[inline]
    fn add(self, other: Colour) -> Colour {
        Colour {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b
        }
    }
}

impl ops::Mul<Colour> for Colour {
    type Output = Colour;

    #[inline]
    fn mul(self, other: Colour) -> Colour {
        Colour {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b
        }
    }
}

macro_rules! colour_mul_impl {
    ($($t:ty)*) => ($(
        impl ops::Mul<$t> for Colour {
            type Output = Colour;

            #[inline]
            fn mul(self, other: $t) -> Colour {
                let v = other as f64;
                Colour { r: self.r * v, g: self.g * v, b: self.b * v}
            }
        }

        impl ops::Mul<Colour> for $t {
            type Output = Colour;

            #[inline]
            fn mul(self, other: Colour) -> Colour {
                let v = self as f64;
                Colour { r: other.r * v, g: other.g * v, b: other.b * v}
            }
        }
    )*)
}

colour_mul_impl!(usize isize i32 u32 i64 u64 f32 f64);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn adding_colours_is_commutative() {
        let a = Colour { r: 0.1, g: 0.2, b: 0.3 };
        let b = Colour { r: 0.01, g: 0.02, b: 0.03 };
        let expected = Colour{ r: 0.11, g: 0.22, b: 0.33 };

        let c1 = a + b;
        assert!(c1.approx_eq(expected), "Expected {:?}, got {:?}", expected, c1);

        let c2 = b + a;
        assert!(c2.approx_eq(expected), "Expected {:?}, got {:?}", expected, c1)

    }
}