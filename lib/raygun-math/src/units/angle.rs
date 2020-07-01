use std::cmp;
use std::marker::PhantomData;
use std::ops;

// ----------------------------------------------------------------------------
// Unit tags
// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum Degrees {}

#[derive(Clone, Copy, Debug)]
pub enum Radians {}

// ----------------------------------------------------------------------------
//
// ----------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct Angle<Unit>(pub f64, PhantomData<Unit>);

impl<Unit> Angle<Unit> {
    pub fn new(x: f64) -> Angle<Unit> {
        Angle(x, PhantomData)
    }
}

impl<Unit> Angle<Unit> {
    pub fn get(&self) -> f64 {
        self.0
    }
}

impl Angle<Degrees> {
    pub fn degrees(self) -> Angle<Degrees> {
        self
    }
    pub fn radians(self) -> Angle<Radians> {
        Angle::new(self.get().to_radians())
    }

    pub fn sin(self) -> f64 {
        self.get().to_radians().sin()
    }
    pub fn cos(self) -> f64 {
        self.get().to_radians().cos()
    }
    pub fn tan(self) -> f64 {
        self.get().to_radians().tan()
    }
}

impl Angle<Radians> {
    pub fn degrees(self) -> Angle<Degrees> {
        Angle::new(self.get().to_degrees())
    }
    pub fn radians(self) -> Angle<Radians> {
        self
    }

    pub fn sin(self) -> f64 {
        self.get().sin()
    }
    pub fn cos(self) -> f64 {
        self.get().cos()
    }
    pub fn tan(self) -> f64 {
        self.get().tan()
    }
}

impl<U> cmp::PartialEq for Angle<U> {
    fn eq(&self, other: &Angle<U>) -> bool {
        self.get() == other.get()
    }
}

impl<U> ops::Add for Angle<U> {
    type Output = Angle<U>;
    fn add(self, other: Angle<U>) -> Angle<U> {
        Angle::new(self.get() + other.get())
    }
}

impl<U> ops::Sub<Angle<U>> for Angle<U> {
    type Output = Angle<U>;
    fn sub(self, other: Angle<U>) -> Angle<U> {
        Angle::new(self.get() - other.get())
    }
}

macro_rules! angle_mul_impl {
    ($($t:ty)*) => ($(
        impl<U> ops::Mul<$t> for Angle<U> {
            type Output = Angle<U>;

            #[inline]
            fn mul(self, other: $t) -> Angle<U> {
                Angle::new(self.get() * (other as f64))
            }
        }

        impl<U> ops::Mul<Angle<U>> for $t {
            type Output = Angle<U>;

            #[inline]
            fn mul(self, other: Angle<U>) -> Angle<U> {
                Angle::new((self as f64) * other.get())
            }
        }
    )*)
}

macro_rules! angle_div_impl {
    ($($t:ty)*) => ($(
        impl<U> ops::Div<$t> for Angle<U> {
            type Output = Angle<U>;

            #[inline]
            fn div(self, other: $t) -> Angle<U> {
                Angle::new(self.get() / (other as f64))
            }
        }

        impl<U> ops::Div<Angle<U>> for $t {
            type Output = Angle<U>;

            #[inline]
            fn div(self, other: Angle<U>) -> Angle<U> {
                Angle::new((self as f64) / other.get())
            }
        }
    )*)
}

angle_mul_impl!(isize usize i32 i64 f32 f64);
angle_div_impl!(isize usize i32 i64 f32 f64);

// -angle
impl<U> ops::Neg for Angle<U> {
    type Output = Angle<U>;
    #[inline]
    fn neg(self) -> Angle<U> {
        Angle::new(-self.get())
    }
}

// ----------------------------------------------------------------------------
//
// ----------------------------------------------------------------------------

pub fn degrees(n: f64) -> Angle<Degrees> {
    Angle::new(n)
}

pub fn radians(n: f64) -> Angle<Radians> {
    Angle::new(n)
}

// ----------------------------------------------------------------------------
//
// ----------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use std::f64::consts;

    fn floats_are_close(a: f64, b: f64, epsilon: f64) -> bool {
        (a - b).abs() < epsilon
    }

    #[test]
    fn mul_angle_by_scalar() {
        let a = degrees(2.0) * 8.0f64;
        assert!(floats_are_close(a.get(), 16.0, 1e-10))
    }

    #[test]
    fn mul_scalar_by_angle() {
        let a = 8.0f64 * degrees(2.0);
        assert!(floats_are_close(a.get(), 16.0, 1e-10))
    }

    #[test]
    fn construct_degrees() {
        let x: Angle<Degrees> = degrees(42.0);
        assert!(floats_are_close(x.get(), 42f64, 1e-10))
    }

    #[test]
    fn sin_of_degrees() {
        let x = degrees(0.0);
        assert!(floats_are_close(x.sin(), 0.0, 1e-10));

        let y = degrees(90.0);
        assert!(floats_are_close(y.sin(), 1.0, 1e-10));

        let z = degrees(45.0);
        assert!(floats_are_close(z.sin(), 0.70710678118, 1e-10));
    }

    #[test]
    fn cos_of_degrees() {
        let x = degrees(0.0);
        assert!(floats_are_close(x.cos(), 1.0, 1e-10));

        let y = degrees(90.0);
        assert!(floats_are_close(y.cos(), 0.0, 1e-10));

        let z = degrees(45.0);
        assert!(floats_are_close(z.cos(), 0.70710678118, 1e-10));
    }

    #[test]
    fn tan_of_degrees() {
        let x = degrees(0.0);
        assert!(floats_are_close(x.tan(), 0f64, 1e-10));

        let y = degrees(90.0);
        let tan_y = y.tan();
        assert!(
            tan_y.is_infinite() || tan_y > 1e16,
            "Expected infinity (or at least a very large number), got {}",
            tan_y
        );

        let z = degrees(45.0);
        assert!(floats_are_close(z.tan(), 1.0, 1e-10));
    }

    #[test]
    fn construct_radians() {
        let x: Angle<Radians> = radians(42.0);
        assert!(floats_are_close(x.get(), 42f64, 1e-10))
    }

    #[test]
    fn sin_of_radians() {
        let x = radians(0.0);
        assert!(floats_are_close(x.sin(), 0.0, 1e-10));

        let y = radians(consts::PI / 2.0);
        assert!(floats_are_close(y.sin(), 1.0, 1e-10));

        let z = radians(consts::PI / 4.0);
        assert!(floats_are_close(z.sin(), 0.70710678118, 1e-10));
    }

    #[test]
    fn cos_of_radians() {
        let x = radians(0.0);
        assert!(floats_are_close(x.cos(), 1.0, 1e-10));

        let y = radians(consts::PI / 2.0);
        assert!(floats_are_close(y.cos(), 0.0, 1e-10));

        let z = radians(consts::PI / 4.0);
        assert!(floats_are_close(z.cos(), 0.70710678118, 1e-10));
    }

    #[test]
    fn tan_of_radians() {
        let x = radians(0.0);
        assert!(floats_are_close(x.tan(), 0f64, 1e-10));

        let y = radians(consts::PI / 2.0);
        let tan_y = y.tan();
        assert!(
            tan_y.is_infinite() || tan_y > 1e16,
            "Expected infinity (or at least a very large number), got {}",
            tan_y
        );

        let z = radians(consts::PI / 4.0);
        assert!(floats_are_close(z.tan(), 1.0, 1e-10));
    }
}
