use crate::geometry::Vector3D;

/// Traingle face of a mesh
#[derive(Debug)]
pub struct Facet {
    vertices: [Vector3D; 3],
}

impl Facet {
    pub fn new(vertices: [Vector3D; 3]) -> Self {
        Self {
            vertices,
        }
    }

    fn translate(&mut self, translation: &Vector3D) {
        for vertex in &mut self.vertices {
            vertex.add(translation);
        }
    }

    /// The lowest z value of all the facet's vertices
    fn lower_z_bound(&self) -> i64 {
        // the unwrap is ok because we know that `vertices` isn't empty
        self.vertices.iter().map(|point| point.z).min().unwrap()
    }

    /// The highest z value of all the facet's vertices
    fn upper_z_bound(&self) -> i64 {
        // the unwrap is ok because we know that `vertices` isn't empty
        self.vertices.iter().map(|point| point.z).max().unwrap()
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

    pub fn to_facet_filter(self) -> FacetFilter {
        FacetFilter::new(self.combined_facets)
    }
}

/// Struct that simply wraps a Facet and caches the results of Facet::lower_z_bound() and Facet::upper_z_bound().
///
/// We convert `Facet`s to `BoundedFacet`s once a scene has been converted to a `FacetFilter`. By that point
/// the facets are no longer part of a mesh and thus won't be moved or otherwise mutated, so we are able to cache
/// the upper/lower z bounds knowing that the bounds won't change.
pub struct BoundedFacet {
    facet: Facet,
    /// Cached value of self.facet.lower_z_bound()
    lower_bound: i64,
    /// Cached value of self.facet.upper_z_bound()
    upper_bound: i64,
}

impl BoundedFacet {
    fn new(facet: Facet) -> Self {
        Self {
            lower_bound: facet.lower_z_bound(),
            upper_bound: facet.upper_z_bound(),
            facet,
        }
    }

    pub fn vertices(&self) -> &[Vector3D; 3] {
        &self.facet.vertices
    }

    pub fn lower_z_bound(&self) -> i64 {
        self.lower_bound
    }

    pub fn upper_z_bound(&self) -> i64 {
        self.upper_bound
    }
}

/// Structure that efficiently filters out the facets that intersect a plane at a given height.
/// Created by [Scene::to_facet_filter].
pub struct FacetFilter {
    /// All facets, sorted by lower bound in descending order
    facets: Vec<BoundedFacet>,
    current_height: i64,
}

impl FacetFilter {
    fn new(facets: Vec<Facet>) -> Self {
        let mut facets: Vec<BoundedFacet> = facets.into_iter().map(BoundedFacet::new).collect();
        // start height is the lowest z value of all the facets' vetexes
        let start_height = facets.iter().map(|facet| facet.lower_bound).min().unwrap();
        // compare b to a so it sorts in descending order
        facets.sort_unstable_by(|a, b| b.lower_bound.cmp(&a.lower_bound));

        Self {
            facets,
            current_height: start_height,
        }
    }

    /// Increases the current height by `increment` and trims facets whose upper bounds
    /// are below the new height (retaining only facets whose upper bounds are at or above
    /// the current height).
    ///
    /// `increment` is unsigned because the height can't be decreased - facets below the
    /// current height have already been trimmed,
    pub fn advance_height(&mut self, increment: u64) {
        self.current_height += increment as i64;
        let current_height = self.current_height;
        self.facets.retain(|facet| facet.upper_bound >= current_height);
    }

    /// Returns an iterator over all facets that intersect with a plane at the current height
    /// (facets whose lower bounds are below the plane and upper bounds are at or above the
    /// plane).
    pub fn intersecting_facets(&self) -> &[BoundedFacet] {
        let first_facet_not_included = self.facets.iter().enumerate().rev()
            .find(|(_, facet)| facet.lower_bound >= self.current_height)
            .map(|(index, _)| index);

        if let Some(index) = first_facet_not_included {
            &self.facets[index + 1..]
        } else {
            &self.facets[..]
        }
    }

    /// Returns true if there are no more facet vertices at or above the current height
    pub fn is_empty(&self) -> bool {
        self.facets.is_empty()
    }

    pub fn current_height(&self) -> i64 {
        self.current_height
    }
}
