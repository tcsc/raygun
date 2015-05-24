use ray::Ray;

///
/// The trait that defines a primitive object
///
pub trait Primitive {
	fn intersects(&self, r: Ray) -> Option<f64>;
}