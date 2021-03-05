#[derive(Debug)]
pub struct Vector3D {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl Vector3D {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self {
            x,
            y,
            z,
        }
    }

    /// Adds `other` to this vector.
    pub fn add(&mut self, other: &Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }

    /// Multiplies each compenent of the vector by the given scalar.
    pub fn mul(&mut self, scalar: i64) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

pub struct Vector2D {
    pub x: i64,
    pub y: i64,
}

impl Vector2D {
    pub fn new(x: i64, y: i64) -> Self {
        Self {
            x,
            y,
        }
    }
}

/// A 2D polygon
pub struct Polygon {
    vertices: Vec<Vector2D>,
}
