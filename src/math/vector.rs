use std::fmt;
use std::cmp;
use std::ops;

/// Defines an immutable 3D vector.
#[derive(Default, Clone, Copy)]
pub struct Vector { x: f64, y: f64, z: f64 }

/// Vector constrcuction function
pub fn vector(x: f64, y: f64, z: f64) -> Vector {
  Vector {x: x, y: y, z: z}
}

impl Vector {
  pub fn cross(&self, other: Vector) -> Vector {
    let x = (self.y * other.z) - (self.z * other.y);
    let y = (self.z * other.x) - (self.x * other.z);
    let z = (self.x * other.y) - (self.y * other.x);
    Vector {x: x, y: y, z: z}
  }

  pub fn dot(&self, other: Vector) -> f64 {
    (self.x * other.x) +  (self.y * other.y) + (self.z * other.z)
  }

  pub fn length(&self) -> f64 {
    let x = self.x * self.x;
    let y = self.y * self.y;
    let z = self.z * self.z;
    (x + y + z).sqrt()
  }

  pub fn normalize(&self) -> Vector {
    let inv_len = 1.0 / self.length();
    Vector { x: self.x * inv_len, y: self.y * inv_len, z: self.z * inv_len }
  }
}

pub fn vector_between(src: Point, dst: Point) -> Vector {
  let x = dst.x - src.x;
  let y = dst.y - src.y;
  let z = dst.z - src.z;
  Vector {x: x, y: y, z: z}
}


impl fmt::Debug for Vector {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {}, {})", self.x, self.y, self.z)
  }
}

impl cmp::PartialEq for Vector {
  fn eq(&self, other: &Vector) -> bool {
    self.x == other.x && self.y == other.y && self.z == other.z
  }

  fn ne(&self, other: &Vector) -> bool {
    self.x != other.x || self.y != other.y || self.z != other.z
  }
}

impl cmp::Eq for Vector {}

impl ops::Add<Vector> for Vector {
  type Output = Vector;

  fn add(self, other: Vector) -> Vector {
    Vector { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
  }
}

impl ops::Sub<Vector> for Vector {
  type Output = Vector;

  fn sub(self, other: Vector) -> Vector {
    Vector { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
  }
}

impl ops::Mul<f64> for Vector {
  type Output = Vector;
  fn mul(self, s: f64) -> Vector {
    Vector { x: self.x * s, y: self.y * s, z: self.z * s }
  }
}

impl ops::Neg for Vector {
  type Output = Vector;

  fn neg(self) -> Vector {
    Vector { x: -self.x, y: -self.y, z: -self.z }
  }
}

// -----------------------------------------------------------------------------
//
// -----------------------------------------------------------------------------

/**
 * An alias for Vector for when you want to emphasize the point-ness of the
 * data, rather than the Vector-ness.
 */
pub type Point = Vector;

/**
 * Point constructor function to avoid the incongrous construction
 * let p : Point = Vector { ... }
 */
pub fn point(x: f64, y: f64, z: f64) -> Point {
  Vector { x: x, y: y, z: z }
}

// -----------------------------------------------------------------------------
//
// -----------------------------------------------------------------------------

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn vector_cross_product() -> () {
    let a = Vector { x: 2.0, y: 3.0, z: 4.0 };
    let b = Vector { x: 5.0, y: 6.0, z: 7.0 };
    assert_eq!(a.cross(b), Vector { x: -3.0, y: 6.0, z: -3.0 })
  }

  #[test]
  fn vector_dot_product() {
    let a = Vector { x: 1.0, y: 2.0, z: 3.0 };
    let b = Vector { x: 4.0, y: 5.0, z: 6.0 };
    assert_eq!(32.0, a.dot(b))
  }

  #[test]
  fn vector_length() {
    assert_eq!((Vector { x: 1.0, y: 0.0, z: 0.0}).length(), 1.0);
    assert_eq!((Vector { x: 0.0, y: 1.0, z: 0.0}).length(), 1.0);
    assert_eq!((Vector { x: 0.0, y: 0.0, z: 1.0}).length(), 1.0);

    let x : f64 = 3.0;
    assert_eq!((Vector { x: 1.0, y: 1.0, z: 1.0}).length(), x.sqrt());
  }

  #[test]
  fn vector_normalise() {
    let v = vector(2.0, 2.0, 2.0);
    let n = v.normalize();
    assert!((n.x - 0.57735).abs() < 0.000001);
    assert!((n.y - 0.57735).abs() < 0.000001);
    assert!((n.z - 0.57735).abs() < 0.000001);
  }

  #[test]
  fn vector_default() -> () {
    let v : Vector = Default::default();
    assert_eq!(v, Vector { x: 0.0, y: 0.0, z: 0.0 });
  }

  #[test]
  fn vector_equality() -> () {
    let a = Vector { x: 1.0, y: 2.0, z: 3.0 };
    assert_eq!(a,a);

    let b = Vector { x: 1.0, y: 2.0, z: 3.0 };
    let c = Vector { x: 3.0, y: 1.0, z: 2.0 };
    assert_eq!(a, b);
    assert!(!(a == c) && !(b == c))
  }
  #[test]
  fn vector_addition() -> () {
    let a = Vector { x: 1.0, y:  3.0, z:  5.0 };
    let b = Vector { x: 7.0, y: 11.0, z: 13.0 };
    assert_eq!(a+b, Vector {x: 8.0, y: 14.0, z: 18.0});
    assert_eq!(a+b, b+a);
  }

  #[test]
  fn vector_subtraction() -> () {
      let a = Vector { x: 1.0, y:  3.0, z:  5.0 };
      let b = Vector { x: 7.0, y: 11.0, z: 13.0 };
      assert_eq!(a-b, Vector {x: -6.0, y: -8.0, z: -8.0});
      assert_eq!(b-a, Vector {x:  6.0, y:  8.0, z:  8.0});
  }

  #[test]
  fn vector_negation() -> () {
    let v = Vector { x: 2.0, y: 4.0, z: 6.0 };
    assert_eq!(-v, Vector { x: -2.0, y: -4.0, z: -6.0 });
    assert_eq!(-v, v * -1.0);
  }

  #[test]
  fn vector_scalar_multiplication() -> () {
    let v = Vector { x: 2.0, y: 4.0, z: 6.0 };
    assert_eq!( v * 1.0, v);
    assert_eq!( v * 2.0, Vector { x: 4.0, y: 8.0, z: 12.0 });
  }

  #[test]
  fn point_construction() -> () {
    let p = point(2.0, 4.0, 6.0);
    assert_eq!(p.x, 2.0);
    assert_eq!(p.y, 4.0);
    assert_eq!(p.z, 6.0);
  }
}
