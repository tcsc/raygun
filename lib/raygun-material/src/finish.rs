#[derive(Debug)]
pub struct Finish {
    // pub opacity: f64,
    pub reflection: f64,
    pub ambient: f64,
    pub diffuse: f64,
    pub highlight_hardness: f64,
}

impl Default for Finish {
    fn default() -> Finish {
        Finish {
            reflection: 0.0,
            ambient: 0.1,
            diffuse: 0.75,
            highlight_hardness: 500.0,
        }
    }
}
