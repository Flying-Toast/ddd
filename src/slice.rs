use crate::geometry::{Polygon, Vector3D, Vector2D};
use crate::mesh::{Scene, BoundedFacet};
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

/// Returns a 2D point which is the result of interpolating `a` along the line segment a---b so that
/// its z coordinate is equal to `plane_z`. Returns `None` if a---b doesn't intersect the z=`plane_z` plane,
/// or if both points are exactly on the plane_z plane.
fn zinterpolate(a_in: &Vector3D, b_in: &Vector3D, plane_z: i64) -> Option<Vector2D> {
    // interpolating a to b can give a different result than b to a, so make sure that whatever order
    // a and b are passed in, they get interpolated in the same way
    let (a, b) = {
        if a_in.pseudo_lt(&b_in) {
            (a_in, b_in)
        } else {
            (b_in, a_in)
        }
    };

    if a.z == b.z {
        // no interpolation if both points are on the same z plane
        None
    } else if a.z == plane_z {
        // a is already on the plane - no interp necessary
        Some(a.to_2d_at_z())
    } else if b.z == plane_z {
        // b is already on the plane - no interp necessary
        Some(b.to_2d_at_z())
    } else if (a.z < plane_z && b.z > plane_z) || (a.z > plane_z && b.z < plane_z) {
        // points are on opposite sides of the plane
        let dx = (b.x - a.x) as f64;
        let dy = (b.y - a.y) as f64;
        let dz = (b.z - a.z) as f64;
        let zdist_aplane = (plane_z - a.z) as f64;
        let ratio = zdist_aplane / dz;
        let mut interpolated = Vector2D::new(a.x, a.y);
        interpolated.x += (dx * ratio) as i64;
        interpolated.y += (dy * ratio) as i64;
        Some(interpolated)
    } else {
        // no intersection exists
        None
    }
}

/// Extracts one group of connected segments from `segments` and stitches them into a polygon.
fn stitch_next(segments: &mut Vec<[Vector2D; 2]>) -> Option<Result<Polygon, Error>> {
    if segments.is_empty() {
        return None;
    }

    if segments.len() < 3 {
        return Some(Err(Error::OpenStitchPolygon));
    }

    let [first_segment_a, first_segment_b] = segments.pop().unwrap();
    let mut builder = Polygon::builder(first_segment_a);

    let mut open_end = first_segment_b;
    loop {
        let mut next_vert_idx: Option<(usize, usize)> = None;
        for (index, segment) in segments.iter().enumerate() {
            if segment[0] == open_end {
                next_vert_idx = Some((index, 1));
                break;
            } else if segment[1] == open_end {
                next_vert_idx = Some((index, 0));
                break;
            }
        }

        if next_vert_idx.is_none() && open_end != *builder.get_start() {
            return Some(Err(Error::OpenStitchPolygon));
        }

        if let Some((index, vertindex)) = next_vert_idx {
            let segment = segments.remove(index);
            builder.line_to(open_end);
            let [v0, v1] = segment;
            if vertindex == 0 {
                open_end = v0;
            } else {
                open_end = v1;
            }
        } else {
            return Some(Ok(builder.close()));
        }
    }
}

/// Turns meshes into [Slice]s
pub struct Slicer<'a> {
    config: &'a SlicerConfig,
}

fn intersect_facets_at_plane(facets: &[BoundedFacet], plane: i64) -> Result<Vec<SliceIsland>, Error> {
    let mut segments = Vec::new();
    for facet in facets {
        let vs = facet.vertices();
        let vertex_combos = &[
            [&vs[0], &vs[1]],
            [&vs[0], &vs[2]],
            [&vs[1], &vs[2]],
        ];
        // dummy array starting values, will get overwritten
        let mut intersections: [Vector2D; 2] = [Vector2D::new(0, 0), Vector2D::new(0, 0)];
        let mut idx = 0;
        let mut have_vertex_on_plane = false;
        for [vertex_a, vertex_b] in vertex_combos {
            if let Some(intersection) = zinterpolate(vertex_a, vertex_b, plane) {
                if vertex_a.z == plane || vertex_b.z == plane {
                    // if the middle vertex lies exactly on the plane, then it will show up
                    // in two interpolations: top---middle, and bottom---middle. To prevent
                    // that same point from being recorded twice, we only add one vertex
                    // that is exactly on the plane (neither of the other two can possibly
                    // be on the plane too, because one has to be above the plane, and the
                    // other has to be below).
                    if have_vertex_on_plane {
                        continue;
                    } else {
                        have_vertex_on_plane = true;
                    }
                }
                assert!(idx < 2, "{:?} intersected more than twice with plane z={}", facet, plane);
                intersections[idx] = intersection;
                idx += 1;
            }
        }
        // idx is 2 because it is still incremented after the last insertion into the array
        assert_eq!(idx, 2, "{:?} didn't have two intersections with plane z={}", facet, plane);
        segments.push(intersections);
    }

    let mut islands = Vec::new();
    while let Some(outline) = stitch_next(&mut segments) {
        islands.push(SliceIsland {
            outline: outline?,
            // TODO: holes. stitch_next() currently treats holes like filled areas
            holes: Vec::new(),
        });
    }

    Ok(islands)
}

impl<'a> Slicer<'a> {
    pub fn new(config: &'a SlicerConfig) -> Self {
        Self { config }
    }

    /// Slices the given scene
    pub fn slice(&self, scene: Scene) -> Result<Vec<Slice>, Error> {
        if scene.is_empty() { return Err(Error::EmptyScene); }
        let mut ff = scene.to_facet_filter();
        let mut slices = Vec::new();

        while !ff.is_empty() {
            let plane = ff.current_height();
            let facets = ff.intersecting_facets();
            let islands = intersect_facets_at_plane(facets, plane)?;
            slices.push(Slice {
                islands,
                thickness: self.config.layer_height,
            });
            ff.advance_height(self.config.layer_height);
        }

        Ok(slices)
    }

}

pub struct SlicerConfig {
    /// Thickness of each printed slice (in nanometers)
    pub layer_height: u64,
}
