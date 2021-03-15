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
/// its z coordinate is equal to `plane_z`. Returns `None` if a---b doesn't intersect the z=`plane_z` plane,
/// or if both points are exactly on the plane_z plane.
fn zinterpolate(a: &Vector3D, b: &Vector3D, plane_z: i64) -> Option<Vector3D> {
    if a.z == b.z {
        // no interpolation if both points are on the same z plane (even if they are
        // both on the plane_z plane)
        None
    } else if a.z == plane_z {
        // a is already on the plane - no interp necessary
        Some(a.clone())
    } else if b.z == plane_z {
        // b is already on the plane - no interp necessary
        Some(b.clone())
    } else if (a.z < plane_z && b.z > plane_z) || (a.z > plane_z && b.z < plane_z) {
        // points are on opposite sides of the plane
        let dx = (b.x - a.x) as f64;
        let dy = (b.y - a.y) as f64;
        let dz = (b.z - a.z) as f64;
        let zdist_aplane = (plane_z - a.z) as f64;
        let ratio = zdist_aplane / dz;
        let mut interpolated = a.clone();
        interpolated.x += (dx * ratio) as i64;
        interpolated.y += (dy * ratio) as i64;
        interpolated.z += (dz * ratio) as i64;
        Some(interpolated)
    } else {
        // no intersection exists
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
            for facet in ff.intersecting_facets() {
                let vs = facet.vertices();
                let vertex_combos = &[
                    [&vs[0], &vs[1]],
                    [&vs[0], &vs[2]],
                    [&vs[1], &vs[2]],
                ];
                for [vertex_a, vertex_b] in vertex_combos {
                    if let Some(intersection) = zinterpolate(vertex_a, vertex_b, ff.current_height()) {
                        //TODO
                    }
                }
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
