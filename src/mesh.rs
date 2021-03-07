use crate::geometry::Vector3D;

/// Traingle face of a mesh
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

    fn translate(&mut self, translation: &Vector3D) {
        for point in &mut self.points {
            point.add(translation);
        }
    }

    /// The lowest z value of all the facet's vertices
    fn lower_z_bound(&self) -> i64 {
        // the unwrap is ok because we know that `points` isn't empty
        self.points.iter().map(|point| point.z).min().unwrap()
    }

    /// The highest z value of all the facet's vertices
    fn upper_z_bound(&self) -> i64 {
        // the unwrap is ok because we know that `points` isn't empty
        self.points.iter().map(|point| point.z).max().unwrap()
    }
}

#[derive(Debug)]
pub struct Mesh {
    facets: Vec<Facet>,
}

impl Mesh {
    pub fn new(facets: Vec<Facet>) -> Self {
        Self { facets }
    }

    pub fn translate(&mut self, translation: &Vector3D) {
        for facet in &mut self.facets {
            facet.translate(&translation);
        }
    }
}

/// One or more [Mesh]es that are sliced/printed together
pub struct Scene {
    /// Every facet of every mesh
    combined_facets: Vec<Facet>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            combined_facets: Vec::new(),
        }
    }

    /// Returns true if there are no meshes in the scene
    pub fn is_empty(&self) -> bool {
        self.combined_facets.is_empty()
    }

    pub fn add_mesh(&mut self, mut mesh: Mesh) {
        self.combined_facets.append(&mut mesh.facets)
    }

    /// Sorts the facets to prepare for slicing
    pub fn zsort_facets(self) -> ZSortedFacets {
        ZSortedFacets::new(self.combined_facets)
    }
}

/// Struct that simply wraps a Facet and saves the results of Facet::lower_z_bound() and Facet::upper_z_bound()
struct CachedFacetBounds {
    facet: Facet,
    /// Cached value of self.facet.lower_z_bound()
    lower_bound: i64,
    /// Cached value of self.facet.upper_z_bound()
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

/// Iterator created by [ZSortedFacets], see [ZSortedFacets::facet_intersections]
pub struct FacetIntersections<'a> {
    facets: std::slice::Iter<'a, CachedFacetBounds>,
}

impl<'a> Iterator for FacetIntersections<'a> {
    type Item = &'a Facet;

    fn next(&mut self) -> Option<Self::Item> {
        self.facets.next().map(|i| &i.facet)
    }
}

/// List of facets of a mesh, sorted by z-height to make slicing more efficient.
/// Created by [Scene::zsort_facets].
pub struct ZSortedFacets {
    /// All facets, sorted by lower bound in descending order
    facets: Vec<CachedFacetBounds>,
    current_height: i64,
}

impl ZSortedFacets {
    fn new(facets: Vec<Facet>) -> Self {
        // start height is the lowest z value of all the facets' vetexes
        let start_height = facets.iter().map(|facet| facet.lower_z_bound()).min().unwrap();
        let mut facets: Vec<CachedFacetBounds> = facets.into_iter().map(CachedFacetBounds::new).collect();
        // compare b to a so it sorts in descending order
        facets.sort_unstable_by(|a, b| b.lower_bound.cmp(&a.lower_bound));

        Self {
            facets,
            current_height: start_height,
        }
    }

    /// Increases the current height by `increment` and trims facets whose upper bounds
    /// are below the new height.
    ///
    /// `increment` is unsigned because the height can't be decreased - facets below the
    /// current height have already been trimmed,
    pub fn advance_height(&mut self, increment: u64) {
        self.current_height += increment as i64;

        // iterator over self.facets indexes (descending order) of facets below the current height
        let mut index_iter = self.facets.iter().enumerate().rev()
            .take_while(|(_, facet)| facet.lower_bound < self.current_height)
            .filter(|(_, facet)| facet.upper_bound < self.current_height)
            .map(|(index, _)| index)
            .peekable();

        let mut ranges: Vec<std::ops::RangeInclusive<usize>> = Vec::new();
        // this loop collapses consecutive indexes into ranges.
        // for example if `index_iter` is [0, 2, 3, 5, 6, 7, 9, 12, 13].rev(), then
        // `ranges` will be filled with [12..=13, 9..=9, 5..=7, 2..=3, 0..=0]
        while let Some(range_end) = index_iter.next() {
            let mut range_start = range_end;
            while let Some(&next) = index_iter.peek() {
                if next == range_start - 1 {
                    range_start = next;
                    let _ = index_iter.next();
                } else {
                    break;
                }
            }
            ranges.push(range_start..=range_end);
        }

        // we use drain to remove the ranges instead of removing by individual indexes,
        // because doing self.facets.remove(index) n times would cause n memcpys, whereas
        // self.facets.drain(0..=n) only causes one memcpy.
        for i in ranges {
            self.facets.drain(i);
        }
    }

    /// Returns an iterator over all facets that intersect with a plane at the current height
    pub fn facet_intersections(&self) -> FacetIntersections {
        let first_facet_above = self.facets.iter().enumerate().rev()
            .find(|(_, facet)| facet.lower_bound > self.current_height)
            .map(|(index, _)| index);

        if let Some(index) = first_facet_above {
            FacetIntersections {
                facets: self.facets[index + 1..].iter(),
            }
        } else {
            FacetIntersections {
                facets: self.facets[..].iter(),
            }
        }
    }

    /// Returns true if there are no more facet vertices at or above the current height
    pub fn done(&self) -> bool {
        self.facets.is_empty()
    }
}
