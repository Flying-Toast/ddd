use std::convert::TryInto;
use crate::geometry::Vector3D;
use crate::mesh::{Facet, Mesh};
use crate::Error;

/// File formats containg mesh data.
///
/// STL files actaully come in 2 different formats: binary and ASCII.
/// Use [detect_stl_type] to determine which kind a given STL is.
#[derive(Debug)]
pub enum FileFormat {
    AsciiStl,
    BinaryStl,
}

/// Measurement units for mesh files.
#[derive(Copy, Clone)]
pub enum MeshFileUnits {
    Inches,
    Millimeters,
}

const MICRONS_PER_INCH: f32 = 25400.0;
const MICRONS_PER_MILLIMETER: f32 = 1000.0;

/// Parses a `Mesh` from the file whose contents are given by `bytes`. `units` is what measurement unit the file uses.
/// All measurements are converted to microns, which is what the rest of the library uses.
pub fn parse_mesh_file(bytes: &[u8], format: FileFormat, units: MeshFileUnits) -> Result<Mesh, Error> {
    match format {
        FileFormat::AsciiStl => AsciiStlParser::new(bytes, units).parse(),
        FileFormat::BinaryStl => BinaryStlParser::new(bytes, units).parse(),
    }
}

/// Detects whether the given STl is ASCII or binary.
/// Returns either `FileFormat::AsciiStl` or `FileFormat::BinaryStl`.
///
/// This assumes that the given STL is either a valid binary STL or a valid ASCII STL - if the STL
/// is not valid, then the output of this function is meaningless. This shouldn't really matter
/// though, because trying to parse an invalid STL will fail anyways.
pub fn detect_stl_type(bytes: &[u8]) -> FileFormat {
    // double-check using bytes.is_ascii, because Solidworks outputs binary STLs with "solid" in the header
    if bytes.starts_with(b"solid") && bytes.is_ascii() {
        FileFormat::AsciiStl
    } else {
        FileFormat::BinaryStl
    }
}

/// Returns true if `coordinate` is finite and non-NaN.
fn is_valid_coordinate(coordinate: f32) -> bool {
    !matches!(coordinate.classify(), std::num::FpCategory::Infinite | std::num::FpCategory::Nan)
}

fn convert_to_microns(value: f32, units: MeshFileUnits) -> f32 {
        match units {
            MeshFileUnits::Inches => value * MICRONS_PER_INCH,
            MeshFileUnits::Millimeters => value * MICRONS_PER_MILLIMETER,
        }
}

struct BinaryStlParser<'a> {
    buf: &'a [u8],
    index: usize,
    facets: Vec<Facet>,
    units: MeshFileUnits,
}

impl<'a> BinaryStlParser<'a> {
    /// Defined by the STL standard
    const HEADER_LENGTH: usize = 80;

    pub fn new(bytes: &'a [u8], units: MeshFileUnits) -> Self {
        Self {
            buf: bytes,
            index: 0,
            facets: Vec::new(),
            units,
        }
    }

    pub fn parse(mut self) -> Result<Mesh, Error> {
        self.eat_header()?;
        let facet_count = self.parse_u32()?;
        if facet_count == 0 {
            return Err(Error::MeshFileParse);
        }
        self.facets.reserve(facet_count as usize);
        for _ in 0..facet_count {
            let facet = self.parse_facet()?;
            self.facets.push(facet);
            // Attributes aren't used in vanilla STL files - we ignore this field
            let _attribute_byte_count = self.parse_u16()?;
        }

        Ok(Mesh::new_zeroed(self.facets))
    }

    /// How many bytes are left in the buffer
    fn bytes_remaining(&self) -> usize {
        self.buf.len() - self.index
    }

    /// Skip the header. Returns `Err` if the header is missing (i.e. the file is smaller than 80 bytes)
    fn eat_header(&mut self) -> Result<(), Error> {
        // STL requires the header
        if self.bytes_remaining() < Self::HEADER_LENGTH {
            Err(Error::MeshFileParse)
        } else {
            self.index += Self::HEADER_LENGTH;
            Ok(())
        }
    }

    /// Parse the next u16 from the buffer
    fn parse_u16(&mut self) -> Result<u16, Error> {
        const NUM_BYTES: usize = std::mem::size_of::<u16>();
        if self.bytes_remaining() < NUM_BYTES {
            return Err(Error::MeshFileParse);
        }
        let bytes: [u8; NUM_BYTES] = self.buf[self.index..self.index + NUM_BYTES]
            .try_into()
            .map_err(|_| Error::MeshFileParse)?;
        self.index += NUM_BYTES;

        Ok(u16::from_le_bytes(bytes))
    }

    /// Parse the next u32 from the buffer
    fn parse_u32(&mut self) -> Result<u32, Error> {
        const NUM_BYTES: usize = std::mem::size_of::<u32>();
        if self.bytes_remaining() < NUM_BYTES {
            return Err(Error::MeshFileParse);
        }
        let bytes: [u8; NUM_BYTES] = self.buf[self.index..self.index + NUM_BYTES]
            .try_into()
            .map_err(|_| Error::MeshFileParse)?;
        self.index += NUM_BYTES;

        Ok(u32::from_le_bytes(bytes))
    }

    /// Parse the next f32 from the buffer, and convert it into microns. Errors if the float is NaN or infinite.
    fn parse_unitized_f32(&mut self) -> Result<f32, Error> {
        const NUM_BYTES: usize = std::mem::size_of::<f32>();
        if self.bytes_remaining() < NUM_BYTES {
            return Err(Error::MeshFileParse);
        }
        let bytes: [u8; NUM_BYTES] = self.buf[self.index..self.index + NUM_BYTES]
            .try_into()
            .map_err(|_| Error::MeshFileParse)?;
        self.index += NUM_BYTES;

        let float = convert_to_microns(f32::from_le_bytes(bytes), self.units);

        if is_valid_coordinate(float) {
            Ok(float)
        } else {
            Err(Error::MeshFileParse)
        }
    }

    /// Parse the next `Vector3D` from the buffer
    fn parse_point(&mut self) -> Result<Vector3D, Error> {
        Ok(Vector3D::new(self.parse_unitized_f32()? as i64, self.parse_unitized_f32()? as i64, self.parse_unitized_f32()? as i64))
    }

    /// Parse the next `Facet` from the buffer
    fn parse_facet(&mut self) -> Result<Facet, Error> {
        let _normal = self.parse_point()?;
        Ok(Facet::new([self.parse_point()?, self.parse_point()?, self.parse_point()?]))
    }
}

/// Parser for ASCII STL files.
struct AsciiStlParser<'a> {
    chars: &'a[u8],
    facets: Vec<Facet>,
    units: MeshFileUnits,
}

impl<'a> AsciiStlParser<'a> {
    pub fn new(chars: &'a[u8], units: MeshFileUnits) -> Self {
        Self {
            chars,
            facets: Vec::new(),
            units,
        }
    }

    pub fn parse(mut self) -> Result<Mesh, Error> {
        self.eat_string(b"solid")?;
        self.eat_line_space()?;

        loop {
            self.eat_string(b"facet normal")?;
            self.eat_whitespace();
            let _normal = self.parse_point()?;
            self.eat_string(b"outer loop")?;
            self.eat_line_space()?;
            let mut points = Vec::with_capacity(3);
            for _ in 0..3 {
                self.eat_string(b"vertex")?;
                self.eat_whitespace();
                points.push(self.parse_point()?);
            }
            // this unwrap is safe because we know the Vec has 3 elements
            let points: [Vector3D; 3] = points.try_into().unwrap();
            self.facets.push(Facet::new(points));
            self.eat_string(b"endloop")?;
            self.eat_line_space()?;
            self.eat_string(b"endfacet")?;
            self.eat_line_space()?;
            if self.peek_check(b"endsolid")? {
                break;
            }
        }

        if self.facets.is_empty() {
            Err(Error::MeshFileParse)
        } else {
            Ok(Mesh::new_zeroed(self.facets))
        }
    }

    /// Eats chars from the buffer as long as they match the contents of `string`. Returns `Err` if they don't match.
    fn eat_string(&mut self, string: &[u8]) -> Result<(), Error> {
        if self.peek_check(string)? {
            self.chars = &self.chars[string.len()..];
            Ok(())
        } else {
            Err(Error::MeshFileParse)
        }
    }

    /// Eats chars until a newline (eats the newline too).
    fn eat_line(&mut self) -> Result<(), Error> {
        while self.eat_char()? != b'\n' {}
        Ok(())
    }

    /// Eats one char.
    fn eat_char(&mut self) -> Result<u8, Error> {
        if !self.chars.is_empty() {
            let ch = self.chars[0];
            self.chars = &self.chars[1..];
            Ok(ch)
        } else {
            Err(Error::MeshFileParse)
        }
    }

    fn eat_whitespace(&mut self) {
        while !self.chars.is_empty() && self.chars[0].is_ascii_whitespace() {
            let _ = self.eat_char();
        }
    }

    fn eat_line_space(&mut self) -> Result<(), Error> {
        self.eat_line()?;
        self.eat_whitespace();
        Ok(())
    }

    /// Checks whether or not the next chars in the buffer match `string`.
    fn peek_check(&self, string: &[u8]) -> Result<bool, Error> {
        if string.len() > self.chars.len() {
           Err(Error::MeshFileParse)
        } else {
            Ok(&self.chars[..string.len()] == string)
        }
    }

    fn parse_point(&mut self) -> Result<Vector3D, Error> {
        let mut coordinates: [f32; 3] = [0.0; 3];
        for i in 0..3 {
            let mut float = String::new();
            while !self.chars.is_empty() && !self.chars[0].is_ascii_whitespace() {
                // this unwrap is safe because we already made sure that `chars` isn't empty
                float.push(self.eat_char().unwrap() as char);
            }
            let coord = convert_to_microns(float.parse().map_err(|_| Error::MeshFileParse)?, self.units);
            if !is_valid_coordinate(coord) {
                return Err(Error::MeshFileParse);
            }
            coordinates[i] = coord;
            self.eat_whitespace();
        }

        Ok(Vector3D::new(coordinates[0] as i64, coordinates[1] as i64, coordinates[2] as i64))
    }
}
