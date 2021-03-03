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

#[derive(Debug)]
pub struct Facet {
    points: [Vector3D; 3],
}

impl Facet {
    pub fn new(points: [Vector3D; 3]) -> Self {
        Self {
            points,
        }
    }

    pub fn translate(&mut self, translation: &Vector3D) {
        for point in &mut self.points {
            point.add(translation);
        }
    }

    /// The lowest corner of the bounding cube around the facet (the corner closest to the origin).
    pub fn lower_bound(&self) -> Vector3D {
        let mut min_x = self.points[0].x;
        let mut min_y = self.points[0].y;
        let mut min_z = self.points[0].z;

        // skip 1 because the first point is already in min_x/min_y/min_z
        for point in self.points.iter().skip(1) {
            min_x = std::cmp::min(point.x, min_x);
            min_y = std::cmp::min(point.y, min_y);
            min_z = std::cmp::min(point.z, min_z);
        }

        Vector3D::new(min_x, min_y, min_z)
    }
}

#[derive(Debug)]
pub struct Mesh {
    facets: Vec<Facet>,
}

impl Mesh {
    /// Creates a new `Mesh` from the given facets and 'zeroes' the mesh (see [to_zeroed](Self::to_zeroed)).
    pub fn new_zeroed(facets: Vec<Facet>) -> Self {
        let mut this = Self {
            facets,
        };
        this.to_zeroed();

        this
    }

    /// Translates the mesh into positive coordinate space.
    ///
    /// The most negative (or least positive) x coordinate will become x=0, etc.
    pub fn to_zeroed(&mut self) {
        let mut translation = self.facets[0].lower_bound();
        for min_point in self.facets.iter().skip(1).map(|facet| facet.lower_bound()) {
            translation.x = std::cmp::min(translation.x, min_point.x);
            translation.y = std::cmp::min(translation.y, min_point.y);
            translation.z = std::cmp::min(translation.z, min_point.z);
        }
        translation.mul(-1);
        self.translate(&translation);
    }

    pub fn translate(&mut self, translation: &Vector3D) {
        for facet in &mut self.facets {
            facet.translate(&translation);
        }
    }
}
