#[derive(Debug)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
        }
    }
}

#[derive(Debug)]
pub struct Facet {
    points: [Point3D; 3],
    normal: Point3D,
}

impl Facet {
    pub fn new(points: [Point3D; 3], normal: Point3D) -> Self {
        Self {
            points,
            normal,
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    facets: Vec<Facet>,
}

impl Mesh {
    pub fn new(facets: Vec<Facet>) -> Self {
        Self {
            facets,
        }
    }
}
