use scene::Scene;
use image::{Rgba, RgbaImage};
use colour::Colour;
use ray::Ray;
use primitive::Primitive;
use math::{Vector, UnitVector, Point, vector, point};
use colour;
use material::Finish;
use light;

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
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let c = trace(projection.ray_for(x, y), scene);
        *pixel = pack_pixel(c)
    }

    Some(img)
}

fn pack_pixel(c: Colour) -> Rgba<u8> {
    Rgba([
        (255.0 * c.r).min(255.0) as u8,
        (255.0 * c.g).min(255.0) as u8,
        (255.0 * c.b).min(255.0) as u8,
        255,
    ])
}

///
/// Describes the intersection of a ray and an object
///
pub struct Intersection<'a> {
    obj: &'a Primitive,
    dist: f64
}

///
/// Finds the object in the scene that intersects closest to the ray origin.
/// Ridiculously inefficient for large scenes, as it just iterates through the
/// objects in a scene and finds the closest thing. Could be made faster by
/// maintaining a BSP- or octree and only doing expensive intersection
/// tests on objects tht could possibly intersect the ray.
///
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

///
/// Specular highlights using the blinn-phong shading model
///
fn blinn_phong_highlight(viewdir: UnitVector,
                         light_ray: Ray,
                         surface_normal: UnitVector,
                         light_colour: Colour,
                         finish: &Finish) -> Colour {
    if !finish.highlight_hardness.is_infinite() {
        let half_vector = (light_ray.dir - viewdir).normalize();
        let intensity = half_vector.dot(surface_normal)
                                   .max(0.0)
                                   .powf(finish.highlight_hardness);

        light_colour * intensity
    } else {
       colour::BLACK
    }
}

///
/// Calculates the light ingfalling on the given point, from all lights in the scene
///
fn light_surface(viewdir: UnitVector,
                 surface_pt: Point,
                 surface_normal: UnitVector,
                 surface_colour: Colour,
                 surface_finish: &Finish,
                 scene: &Scene) -> Colour {

    let mut result = surface_colour * surface_finish.ambient;
    for light in scene.lights.iter() {
        if let Some(light_colour) = light.lights(surface_pt) {
            let light_beam = light.src() - surface_pt;

            // if the light beam is not behind the point we're trying to light...
            if light_beam.dot(surface_normal) > 0.0 {
                // define a ray pointing from the surface to the light source
                let pp = surface_pt + (1e-6 * surface_normal);
                let light_ray = Ray::new(pp, light_beam.normalize());

                if !is_shadowed(light_ray, light_beam.length(), scene) {
                    // compute the diffuse lighting
                    let lambert_coeff = light_ray.dir.dot(surface_normal);
                    let diffuse = surface_finish.diffuse * surface_colour * light_colour * lambert_coeff;

                    // compute the specular highlight
                    let specular = blinn_phong_highlight(viewdir, light_ray, surface_normal, light_colour, surface_finish);
                    result = result + diffuse + specular;
                }
            }
        }
    }
    result
}

fn is_shadowed(light_ray: Ray, light_distance: f64, scene: &Scene) -> bool {
    if let Some(ix) = closest_intersecting_object(light_ray, scene) {
        ix.dist < light_distance
    }
    else {
        false
    }
}

///
/// Traces a ray from the source pixel through the scene
///
fn trace(r: Ray, scene: &Scene) -> Colour {
    // loop {
        if let Some(ix) = closest_intersecting_object(r, scene) {
            let surface_point = r.extend(ix.dist);
            let surface_normal = ix.obj.normal(surface_point);
            let surface_colour = colour::WHITE;
            light_surface(r.dir, surface_point,
                                 surface_normal,
                                 surface_colour,
                                 &Finish::default(),
                                 scene)
        }
        else {
            colour::BLACK
        }
    //}
}


#[cfg(test)]
mod test {
    use super::*;
    use colour;
    use primitive::Sphere;
    use scene::Scene;
    use math::{point, vector, Vector};
    use ray::Ray;

    fn test_scene() -> Scene {
        let mut s = Scene::new();
        s.add_objects(vec!(
            Box::new(Sphere::new(point(0.0, 0.0, 0.0), 1.0)),
            Box::new(Sphere::new(point(0.0, 0.0, 1.0), 1.0))
        ));
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
            assert!(floats_are_close(9.0, i.dist, 1e-6))
        } else {
            panic!("Expected an intersecting object")
        }
    }

    #[test]
    fn non_intersecting_ray_finds_nothing() {
        let s = test_scene();
        let r = Ray::new(point(0.0, 0.0, -10.0), vector(0.0, 1.0, 1.0));
        match super::closest_intersecting_object(r, &s) {
            None => {},
            Some(_) => panic!("Expected an intersecting object")
        }
    }

    #[test]
    fn un_occluded_light_is_not_shadowed() {
        let mut s = Scene::new();
        let light_loc = point(100.0, 100.0, 100.0);
        s.add_point_light(light_loc, colour::WHITE);

        let surface_pt = point(0.0, 0.0, 0.0);
        let light_beam = Vector::between(surface_pt, light_loc);
        let light_ray = Ray::new(surface_pt, light_beam.normalize());

        assert!(!super::is_shadowed(light_ray, light_beam.length(), &s))
    }

    #[test]
    fn occluded_light_is_shadowed() {
        let mut s = Scene::new();
        let light_loc = point(100.0, 100.0, 100.0);
        s.add_point_light(light_loc, colour::WHITE);
        s.add_object(Box::new(Sphere::new(point(90.0, 90.0, 90.0), 2.0)));

        let surface_pt = point(0.0, 0.0, 0.0);
        let light_beam = Vector::between(surface_pt, light_loc);
        let light_ray = Ray::new(surface_pt, light_beam.normalize());

        assert!(super::is_shadowed(light_ray, light_beam.length(), &s))
    }

    #[test]
    fn objects_on_other_side_of_light_do_not_occlude_light() {
        let mut s = Scene::new();
        let light_loc = point(100.0, 100.0, 100.0);
        s.add_point_light(light_loc, colour::WHITE);
        s.add_object(Box::new(
            Sphere::new(point(110.0, 110.0, 11.0), 2.0)
        ));

        let surface_pt = point(0.0, 0.0, 0.0);
        let light_beam = Vector::between(surface_pt, light_loc);
        let light_ray = Ray::new(surface_pt, light_beam.normalize());

        assert!(!super::is_shadowed(light_ray, light_beam.length(), &s))
    }
}
