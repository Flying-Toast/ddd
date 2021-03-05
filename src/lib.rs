pub mod geometry;
/// Parsing logic for different 3D file formats
pub mod parsing;
pub mod mesh;

#[derive(Debug)]
pub enum Error {
    /// Error parsing a mesh file (STL, OBJ, etc)
    MeshFileParse,
}
