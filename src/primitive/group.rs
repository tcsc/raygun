use math::{Matrix, Point, Vector, IDENTITY};
use primitive::Primitive;
use ray::Ray;

pub struct Group {
    transform: Matrix,
    children: Vec<Box<Primitive>>
}

impl Group {

}

impl Primitive for Group {
    fn intersects(&self, r: Ray) -> Option<f64> {
        None
    }

    fn normal(&self, pt: Point) -> Vector {
        Vector::default()
    }
}

impl Default for Group {
    fn default() -> Group {
        Group {
            transform: IDENTITY,
            children: Vec::new()
        }
    }
}

#[cfg(test)]
mod test {
    use super::Group;
    use primitive::Primitive;
    use math::{point, vector, IDENTITY};
    use ray::Ray;

    #[test]
    fn default() {
        let g = Group::default();
        assert_eq!(g.transform, IDENTITY);
        assert_eq!(g.children.len(), 0);
    }

    #[test]
    fn nothing_intersects() {
        let g = Group::default();
        let r = Ray::new(point(0.0, 0.0, 0.0), vector(0.0, 1.0, 0.0));
        assert_eq!(g.intersects(r), None);
    }
}