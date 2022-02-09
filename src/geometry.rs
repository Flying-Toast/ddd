#[derive(Debug, Clone)]
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

    /// Creates a 2D vector of this 3D vector without the z
    pub(crate) fn to_2d_at_z(&self) -> Vector2D {
        Vector2D::new(self.x, self.y)
    }

    /// Returns whether this vector is 'less than' `other`. 'less than' doens't really mean anything,
    /// it's just used to enable multiple vectors to be ordered deterministically (see crate::slice::zinterpolate)
    pub(crate) fn pseudo_lt(&self, other: &Self) -> bool {
        if self.x != other.x {
            self.x < other.x
        } else if self.y != other.y {
            self.y < other.y
        } else if self.z != other.z {
            self.z < other.z
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

/// A closed 2D polygon
#[derive(Debug)]
pub struct Polygon {
    vertices: Vec<Vector2D>,
}

impl Polygon {
    /// Creates a Polygon builder. `start` is the first vertex of the polygon.
    pub fn builder(start: Vector2D) -> PolygonBuilder {
        PolygonBuilder::new(start)
    }

    /// The vertices of this polygon
    pub fn vertices(&self) -> &[Vector2D] {
        &self.vertices
    }
}

/// Builds a closed polygon.
/// New `PolygonBuilders` are created using [Polygon::builder()](Polygon::builder).
pub struct PolygonBuilder {
    vertices: Vec<Vector2D>,
    start_point: Vector2D,
}

impl PolygonBuilder {
    fn new(start: Vector2D) -> Self {
        Self {
            vertices: vec![start.clone()],
            start_point: start,
        }
    }

    /// Adds a line from the end of the previous point (or from the start point, if this is the first line)
    /// to the point `to`.
    pub fn line_to(&mut self, to: Vector2D) {
        self.vertices.push(to);
    }

    /// Adds a final line to the start point, then builds the Polygon.
    pub fn close(self) -> Polygon {
        let Self {
            mut vertices,
            start_point,
        } = self;
        vertices.push(start_point);

        Polygon {
            vertices,
        }
    }

    pub fn get_start(&self) -> &Vector2D {
        &self.start_point
    }
}
