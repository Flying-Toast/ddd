#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
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
    points: [Point; 3],
    normal: Point,
}

impl Facet {
    pub fn new(points: [Point; 3], normal: Point) -> Self {
        Self {
            points,
            normal,
        }
    }
}
