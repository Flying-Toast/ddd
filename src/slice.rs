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
        // no interpolation if both points are on the same z plane
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
        let mut interpolated = Vector3D::new(a.x, a.y, plane_z);
        interpolated.x += (dx * ratio) as i64;
        interpolated.y += (dy * ratio) as i64;
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
            let plane = ff.current_height();
            for facet in ff.intersecting_facets() {
                let vs = facet.vertices();
                let vertex_combos = &[
                    [&vs[0], &vs[1]],
                    [&vs[0], &vs[2]],
                    [&vs[1], &vs[2]],
                ];
                let mut intersections = Vec::with_capacity(2);
                let mut have_vertex_on_plane = false;
                for [vertex_a, vertex_b] in vertex_combos {
                    if let Some(intersection) = zinterpolate(vertex_a, vertex_b, plane) {
                        if vertex_a.z == plane || vertex_b.z == plane {
                            // if the middle vertex lies exactly on the plane, then it will show up
                            // in two interpolations: top---middle, and bottom---middle. To prevent
                            // that same point from being in `intersections` twice, we only add one
                            // vertex that is exactly on the plane (neither of the other two can
                            // possibly be on the plane too, because one has to be above the plane,
                            // and the other has to be below).
                            if have_vertex_on_plane {
                                continue;
                            } else {
                                have_vertex_on_plane = true;
                            }
                        }
                        intersections.push(intersection);
                    }
                }
            }

            ff.advance_height(self.config.layer_height);
        }

        todo!();
    }
}

pub struct SlicerConfig {
    /// Thickness of each printed slice (in nanometers)
    pub layer_height: u64,
}
