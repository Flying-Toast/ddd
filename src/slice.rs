use crate::geometry::{Polygon, Vector3D};
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
    /// The thickness (in nanometers) of this slice (the "layer height")
    thickness: u64,
    islands: Vec<SliceIsland>,
}

/// Returns a new point which is the result of interpolating `a` along the line segment a---b so that
/// its z coordinate is equal to `plane_z`. Returns `None` if a---b doesn't intersect the z=`plane_z` plane.
fn zinterpolate(a: &Vector3D, b: &Vector3D, plane_z: i64) -> Option<Vector3D> {
    // if either point is already on the plane, there is no interpolation to do
    if a.z == plane_z { return Some(a.clone()); }
    if b.z == plane_z { return Some(b.clone()); }

    // check if the points are on opposite sides of the plane
    let (below, above);
    if (a.z < plane_z && b.z > plane_z) || (a.z > plane_z && b.z < plane_z) {
        todo!();
    } else {
        None
    }
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
                //TODO:
                // interpolate the vertices
                // make sure to handle case where some vertices are exactly on the plane
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
