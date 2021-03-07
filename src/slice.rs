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
    fn new(config: &'a SlicerConfig) -> Self {
        Self { config }
    }

    /// Slices the given scene
    fn slice(&self, scene: Scene) -> Result<Vec<Slice>, Error> {
        let mut facets = scene.zsort_facets();

        while !facets.is_empty() {

            facets.advance_height(self.config.layer_height);
        }

        todo!();
    }
}

pub struct SlicerConfig {
    /// Thickness of each printed slice
    pub layer_height: u64,
}
