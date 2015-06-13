pub struct Highlight {
	pub intensity: f64,
	pub size: f64
}

pub struct Finish {
	pub opacity: f64,
	pub reflection: f64,
	pub ambient: f64,
	pub diffuse: f64,
	pub highlight: Highlight
}

impl Default for Finish {
	fn default() -> Finish {
		Finish {
	        opacity: 1.0,
	        reflection: 0.0,
	        ambient: 0.1,
	        diffuse: 0.75,
	        highlight: Highlight { intensity: 0.9, size: 60.0 }
	    }
	}
}