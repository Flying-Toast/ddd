use crate::geometry::Polygon;
use crate::mesh::Scene;
use crate::Error;

/// A single closed polygon in a slice. One slice can contain multiple closed polygons that aren't connected.
#[derive(Debug)]
pub struct SliceIsland {
    outline: Polygon,
    /// Negative spaces inside the `outline`
    holes: Vec<Polygon>,
}

/// A single layer of a sliced mesh. Composed of multiple `SliceIsland`s.
#[derive(Debug)]
pub struct Slice {
    /// The thickness (in microns) of this slice (the "layer height")
    thickness: u64,
    islands: Vec<SliceIsland>,
}

/// Turns meshes into [Slice]s
pub struct Slicer<'a> {
    config: &'a SlicerConfig,
}

impl<'a> Slicer<'a> {
    pub fn new(config: &'a SlicerConfig) -> Self {
        Self { config }
    }

    /// Slices the given scene
    pub fn slice(&self, scene: Scene) -> Result<Vec<Slice>, Error> {
        if scene.is_empty() { return Err(Error::EmptyScene); }
        let mut ff = scene.to_facet_filter();

        while !ff.is_empty() {
            for facet in ff.intersections() {
                let (vertices_below, vertices_above): (Vec<_>, Vec<_>) = facet.vertices().iter()
                    .partition(|&vertex| vertex.z < ff.current_height());
                //TODO: handle case where some vertices are exactly one the plane
            }

            ff.advance_height(self.config.layer_height);
        }

        todo!();
    }
}

pub struct SlicerConfig {
    /// Thickness of each printed slice
    pub layer_height: u64,
}
