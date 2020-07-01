use std::sync::Arc;

use nom::{branch::alt, combinator::map, multi::many0, IResult};

use raygun_primitives::Object;

use crate::{constructs::*, SceneRef};

mod r#box;
mod plane;
mod point_light;
mod sphere;
mod union;

fn primitive<'a>(scene: SceneRef) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Arc<Object>> {
    let p = ws(alt((
        sphere::parse(scene.clone()),
        r#box::parse(scene.clone()),
        plane::parse(scene.clone()),
        point_light::parse(scene.clone()),
        union::parse(scene.clone()),
    )));

    map(p, Arc::new)
}

pub fn primitives<'a>(scene: SceneRef) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Vec<Arc<Object>>> {
    many0(primitive(scene))
}

#[cfg(test)]
mod test {
    //use super::*;
}
