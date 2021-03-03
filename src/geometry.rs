/// A 3D point. Coordinates are in microns.
#[derive(Debug)]
pub struct Point3D {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Point3D {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
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
