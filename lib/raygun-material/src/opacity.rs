#[derive(Debug)]
pub struct Opacity {
    pub alpha: f64,
    pub refractive_index: f64,
}

impl Default for Opacity {
    fn default() -> Self {
        Opacity {
            alpha: 1.0,
            refractive_index: 1.0,
        }
    }
}
