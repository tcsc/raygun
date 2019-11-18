use rayon;

use crate::{
    colour::{self, Colour},
    math::{Vector, UnitVector, Point, point},
    primitive::Object,
    ray::Ray,
    scene::{Scene, LightInfo},
    material::Finish,
};

use image::{Rgba, RgbaImage};
use log::{debug, error};

pub struct RenderOptions {
    pub height: isize,
    pub width: isize,
}

pub fn render(scene: &Scene, options: RenderOptions) -> Option<RgbaImage> {
    use std::sync::mpsc::channel;

    debug!("Collecting lights...");

    let lights = &scene.lights();

    debug!("Found {} lights in scene", lights.len());

    debug!("Beginning trace...");

    let pixel_count = options.width * options.height;
    let img = rayon::scope(move |s| {
        let mut img = RgbaImage::new(
            options.width as u32,
            options.height as u32);

        let projection = scene.camera.projector(options.width,
                                                options.height);

        let (tx, rx) = channel();

        debug!("Spawning render tasks...");

        for y in 0..(options.height as u32) {
            for x in 0..(options.width as u32) {
                let ray = projection.ray_for(x, y);
                let sender = tx.clone();
                s.spawn(move |_| {
                    let c = trace(ray, scene, lights);
                    sender.send((x, y, c)).unwrap();
                })
            }
        }

        debug!("Gathering pixels...");

        for _ in 0..pixel_count {
            let (x, y, colour) = rx.recv().unwrap();
            img.put_pixel(x, y, pack_pixel(colour));
        }

        img
    });

    debug!("Trace complete.");

    Some(img)
}

fn distance(a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dz = b.z - a.z;
    ((dx * dx) + (dy * dy) + (dz * dz)).sqrt()
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
    obj: &'a Object,
    dist: f64,
    point: Point,
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

    let mut min_dist = f64::INFINITY;
    let mut intersecting_obj = None;
    let mut intersection_point = point(f64::NAN, f64::NAN, f64::NAN);

    for obj in scene.objects.iter() {
        if let Some(pt) = obj.intersects(r) {
            let n = distance(r.src, pt);
            if n < min_dist {
                min_dist = n;
                intersecting_obj = Some(obj);
                intersection_point = pt;
            }
        }
    }
    intersecting_obj.map(|o| Intersection{
        obj: &(*o),
        dist: min_dist,
        point: intersection_point
    })
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
/// Calculates the light falling on the given point, from all lights in the scene
///
fn light_surface(viewdir: UnitVector,
                 surface_pt: Point,
                 surface_normal: UnitVector,
                 surface_colour: Colour,
                 surface_finish: &Finish,
                 scene: &Scene,
                 lights: &Vec<LightInfo>) -> Colour {

    let mut result = surface_colour * surface_finish.ambient;
    for light_info in lights.iter() {
        let light = light_info.light.as_light().unwrap();
        let point_in_light_space = light_info.transform.matrix * surface_pt;

        if let Some(light_colour) = light.illuminates(point_in_light_space) {
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

/// Reflect the incoming ray at the point of intersection, moving it
/// infinitesimally back along the incoming ray so as not to immediately
/// find the same point on the same object again.
fn reflect(inbound: Ray, pt: Point, normal: Vector) -> Ray {
    let reflected = inbound.reflect(normal, pt);
    let offset = normal * 1e-12;
    Ray {
        src: reflected.src + offset,
        dir: reflected.dir
    }
}


///
/// Traces a ray from the ray source through the scene
///
fn trace(inbound_ray: Ray, scene: &Scene, lights: &Vec<LightInfo>) -> Colour {
    use std::collections::VecDeque;

    const THRESHOLD : f64 = 1e-12;

    let mut contribs = Vec::new();
    let mut rays = VecDeque::new();
    rays.push_back((inbound_ray, 1.0));

    while !rays.is_empty() {
        let (ray, weight) = rays.pop_front().unwrap();
        let intersection = closest_intersecting_object(ray, scene);
        let contrib = match intersection {
            Some(ix) => {
                let surface_point = ix.point;
                let surface = ix.obj.surface_at(surface_point);
                let colour = light_surface(ray.dir,
                                           surface_point,
                                           surface.normal,
                                           surface.colour,
                                           &surface.finish,
                                           scene,
                                           lights);

                if surface.finish.reflection > 0.0 {
                    let new_weight = weight * surface.finish.reflection;
                    if new_weight > THRESHOLD {
                        let new_ray = reflect(ray, surface_point, surface.normal);
                        rays.push_back((new_ray, new_weight));
                    }
                }

                colour
            },
            None => {
                scene.sky(ray)
            }
        };

        contribs.push(contrib * weight);
    }

    contribs.iter().fold(colour::BLACK, |sum, b| sum + (*b))
}


#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    use crate::{
        colour,
        primitive::{Object, Primitive, Sphere},
        scene::Scene,
        math::{point, vector, Vector},
        light::PointLight,
        ray::Ray
    };

    fn to_obj<P: Primitive>(p: P) -> Object {
        Object::from(Arc::new(p))
    }

    fn test_scene() -> Scene {
        let mut s = Scene::new();
        let objs = vec!(to_obj(Sphere::new(point(0.0, 0.0, 0.0), 1.0)),
                        to_obj(Sphere::new(point(0.0, 0.0, 1.0), 1.0)));
        for obj in objs {
            s.add_object(obj)
        }

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

        let surface_pt = point(0.0, 0.0, 0.0);
        let light_beam = Vector::between(surface_pt, light_loc);
        let light_ray = Ray::new(surface_pt, light_beam.normalize());

        assert!(!super::is_shadowed(light_ray, light_beam.length(), &s))
    }

    #[test]
    fn occluded_light_is_shadowed() {
        let mut s = Scene::new();
        let light_loc = point(100.0, 100.0, 100.0);
        s.add_object(to_obj(Sphere::new(point(90.0, 90.0, 90.0), 2.0)));

        let surface_pt = point(0.0, 0.0, 0.0);
        let light_beam = Vector::between(surface_pt, light_loc);
        let light_ray = Ray::new(surface_pt, light_beam.normalize());

        assert!(super::is_shadowed(light_ray, light_beam.length(), &s))
    }

    #[test]
    fn objects_on_other_side_of_light_do_not_occlude_light() {
        let mut s = Scene::new();
        let light_loc = point(100.0, 100.0, 100.0);

        s.add_object(
            to_obj(Sphere::new(point(110.0, 110.0, 11.0), 2.0))
        );

        let surface_pt = point(0.0, 0.0, 0.0);
        let light_beam = Vector::between(surface_pt, light_loc);
        let light_ray = Ray::new(surface_pt, light_beam.normalize());

        assert!(!super::is_shadowed(light_ray, light_beam.length(), &s))
    }
}
