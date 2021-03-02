pub mod geometry;
/// Parsing logic for different 3D file formats
pub mod parsing;

#[derive(Debug)]
pub enum Error {
    /// Error parsing a mesh file (STL, OBJ, etc)
    MeshFileParse,
}
