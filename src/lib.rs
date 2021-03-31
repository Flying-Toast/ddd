pub mod geometry;
/// Parsing logic for different 3D file formats
pub mod parsing;
pub mod mesh;
pub mod slice;
pub mod gcode;

#[derive(Debug)]
pub enum Error {
    /// Error parsing a mesh file (STL, OBJ, etc)
    MeshFileParse,
    /// Attempted to slice a scene with no meshes in it
    EmptyScene,
    /// Tried to stitch a set of segments that formed a non-closed polygon
    OpenStitchPolygon,
}
