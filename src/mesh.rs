use crate::geometry::Vector3D;

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

    pub fn points(&self) -> &[Vector3D; 3] {
        &self.points
    }

    /// The lowest corner of the bounding cube around the facet.
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

    /// The lowest z value of all the facet's vertices
    pub fn lower_z_bound(&self) -> i64 {
        // the unwrap is ok because we know that `points` isn't empty
        self.points.iter().map(|point| point.z).min().unwrap()
    }

    /// The highest z value of all the facet's vertices
    pub fn upper_z_bound(&self) -> i64 {
        // the unwrap is ok because we know that `points` isn't empty
        self.points.iter().map(|point| point.z).max().unwrap()
    }
}

#[derive(Debug)]
pub struct Mesh {
    facets: Vec<Facet>,
}

impl Mesh {
    /// Creates a new `Mesh` from the given facets and 'zeroes' the
    /// mesh (translates it into positive coordinate space).
    /// The most negative (or least positive) x coordinate will become x=0, etc.
    pub fn new_zeroed(facets: Vec<Facet>) -> Self {
        let mut this = Self {
            facets,
        };
        this.zeroize();

        this
    }

    fn zeroize(&mut self) {
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

    fn lower_z_bound(&self) -> i64 {
        self.facets.iter().map(|facet| facet.lower_z_bound()).min().unwrap()
    }

    pub fn zsort_facets(self) -> ZSortedFacets {
        let lower_z_bound = self.lower_z_bound();
        ZSortedFacets::new(self.facets, lower_z_bound)
    }
}

struct CachedFacetBounds {
    facet: Facet,
    lower_bound: i64,
    upper_bound: i64,
}

impl CachedFacetBounds {
    fn new(facet: Facet) -> Self {
        Self {
            lower_bound: facet.lower_z_bound(),
            upper_bound: facet.upper_z_bound(),
            facet,
        }
    }
}

pub struct FacetIntersections<'a> {
    facets: std::slice::Iter<'a, CachedFacetBounds>,
}

impl<'a> FacetIntersections<'a> {
    fn new(facets: &'a [CachedFacetBounds]) -> Self {
        Self {
            facets: facets.iter(),
        }
    }
}

impl<'a> Iterator for FacetIntersections<'a> {
    type Item = &'a Facet;

    fn next(&mut self) -> Option<Self::Item> {
        self.facets.next().map(|i| &i.facet)
    }
}

pub struct ZSortedFacets {
    /// All facets, sorted by lower bound in descending order
    facets: Vec<CachedFacetBounds>,
    current_height: i64,
}

impl ZSortedFacets {
    fn new(facets: Vec<Facet>, start_height: i64) -> Self {
        let mut facets: Vec<CachedFacetBounds> = facets.into_iter().map(CachedFacetBounds::new).collect();
        // compare b to a so it sorts in descending order
        facets.sort_unstable_by(|a, b| b.lower_bound.cmp(&a.lower_bound));

        let mut this = Self {
            facets,
            current_height: start_height,
        };
        // advance height by 0 to trim any facets that are already below the start height
        this.advance_height(0);

        this
    }

    /// Increases the current height by `increment` and trims facets whose upper bounds
    /// are below the new height.
    ///
    /// `increment` is unsigned because the height can't be decreased - facets below the
    /// current height have already been trimmed,
    pub fn advance_height(&mut self, increment: u64) {
        self.current_height += increment as i64;
        for i in self.facets.iter().enumerate().rev()
            .take_while(|(_, facet)| facet.lower_bound < self.current_height)
            .filter(|(_, facet)| facet.upper_bound < self.current_height)
            .map(|(index, _)| index)
        {
            todo!("trim");
        }
    }

    /// Returns an iterator over all facets that intersect with a plane at the current height
    pub fn facet_intersections(&self) -> FacetIntersections {
        let first_facet_above = self.facets.iter().enumerate().rev()
            .find(|(_, facet)| facet.lower_bound > self.current_height)
            .map(|(index, _)| index);

        if let Some(index) = first_facet_above {
            FacetIntersections::new(&self.facets[index + 1..])
        } else {
            FacetIntersections::new(&self.facets[..])
        }
    }

    /// Returns true if there are no more facet vertices at or above the current height
    pub fn done(&self) -> bool {
        self.facets.is_empty()
    }
}
