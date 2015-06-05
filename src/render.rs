use scene::Scene;
use image::RgbaImage;
use ray::Ray;
use math::{Vector, UnitVector, Point};

pub struct RenderOptions {
    pub height: isize,
    pub width: isize,
}

impl RenderOptions {
    fn new() -> RenderOptions {
        RenderOptions {
            width:   0,
            height:  0
        }
    }
}

pub fn render(scene: &Scene, options: RenderOptions) -> Option<RgbaImage> {
    let mut img = RgbaImage::new(
        options.width as u32,
        options.height as u32);

    let projection = scene.camera.projector(options.width, options.height);
    for y in 0 .. options.height {
        for x in 0 .. options.width {
            let r = projection.ray_for(x, y);
        }
    }

    Some(img)
}


#[cfg(test)]
mod test {

}
