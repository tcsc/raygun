use math::{Point, Vector};

///
/// Represents a ray through the scene, starting at `src` and heading along
/// `dir`. The vector is always normalised.
///
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Ray {
	pub src: Point,
	pub dir: Vector
}

impl Ray {
	/// Initialises a new `Ray` instance, normalising the supplied vector
	/// during construction.
	pub fn new(src: Point, dir: Vector) -> Ray {
		Ray {
			src: src,
			dir: dir.normalize()
		}
	}

	/// Calculates the point `len` units along the ray
	pub fn extend(&self, len: f64) -> Point {
        self.src + (self.dir * len)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use math::{vector, point};

	#[test]
	fn ray_construction() {
		let r = Ray::new(point(1.0, 2.0, 3.0), vector(2.0, 2.0, 2.0));

		assert_eq!(r.src, point(1.0, 2.0, 3.0));
		assert_eq!(r.dir, vector(2.0, 2.0, 2.0).normalize());
	}

	#[test]
	fn extention() {
		let r = Ray::new(point(0.0, 0.0, 0.0), vector(0.0, 1.0, 0.0));
		let pt = r.extend(10.0);
		assert_eq!(pt, point(0.0, 10.0, 0.0))
	}
}