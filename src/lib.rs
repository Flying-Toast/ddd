pub mod geometry;
/// Parsing logic for different 3D file formats
pub mod parsing;

#[derive(Debug)]
pub enum Error {
    StlParse,
}
