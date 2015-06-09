use scene::Scene;
use image::RgbaImage;
use ray::Ray;
use primitive::Primitive;
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

///
/// A colour value, with each channel normalised between 0 and 1
///
struct Colour { r: f64, g: f64, b: f64 }

///
/// Describes the intersection of a ray and an object
///
pub struct Intersection<'a> {
    obj: &'a Primitive,
    dist: f64
}

fn closest_intersecting_object<'a>(r: Ray, scene: &'a Scene) -> Option<Intersection<'a>> {
    use std::f64;

    let mut dist = f64::INFINITY;
    let mut obj = None;

    for p in scene.objects.iter() {
        if let Some(n) = p.intersects(r) {
            if n < dist {
                dist = n;
                obj = Some(p);
            }
        }
    }
    obj.map(|o| Intersection{ obj: &(**o), dist: dist} )
}

fn trace(r: Ray, scene: &Scene) -> Colour {
    Colour {r: 0.0, g: 0.0, b: 0.0}
}


#[cfg(test)]
mod test {
    use super::*;
    use primitive::Sphere;
    use scene::Scene;
    use math::{point, vector};
    use ray::Ray;

    fn test_scene() -> Scene {
        let mut s = Scene::new();
        s.add(Sphere::new(point(0.0, 0.0, 0.0), 1.0));
        s.add(Sphere::new(point(0.0, 0.0, 1.0), 1.0));
        s
    }

    fn floats_are_close(a: f64, b: f64, epsilon: f64) -> bool {
        (a-b).abs() < epsilon
    }

    #[test]
    fn closest_intersecting_object_found() {
        let s = test_scene();
        let r = Ray::new(point(0.0, 0.0, -10.0), vector(0.0, 0.0, 1.0));
        if let Some(i) = super::closest_intersecting_object(r, &s) {
            println!("dist: {}", i.dist);
            assert!(floats_are_close(9.0, i.dist, 1e-6))
        } else {
            panic!("Expected an intersecting object")
        }
    }

    #[test]
    fn non_intersecting_ray_finds_nothong() {
        let s = test_scene();
        let r = Ray::new(point(0.0, 0.0, -10.0), vector(0.0, 1.0, 1.0));
        match super::closest_intersecting_object(r, &s) {
            None => {},
            Some(_) => panic!("Expected an intersecting object")
        }
    }
}
