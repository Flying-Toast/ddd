use std::convert::TryInto;
use crate::geometry::{Point, Facet, Mesh};
use crate::Error;

/// Parses a `Mesh` from a binary STL file. The contents of the file are given by `bytes`.
pub fn parse_binary_stl(bytes: &[u8]) -> Result<Mesh, Error> {
    BinaryStlParser::new(bytes).parse()
}

struct BinaryStlParser<'a> {
    buf: &'a [u8],
    index: usize,
    facets: Vec<Facet>,
}

impl<'a> BinaryStlParser<'a> {
    /// Defined by the STL standard
    const HEADER_LENGTH: usize = 80;

    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            buf: bytes,
            index: 0,
            facets: Vec::new(),
        }
    }

    pub fn parse(mut self) -> Result<Mesh, Error> {
        self.eat_header()?;
        let facet_count = self.parse_u32()?;
        for _ in 0..facet_count {
            let facet = self.parse_facet()?;
            self.facets.push(facet);
            // Attributes aren't used in vanilla STL files - we ignore this field
            let _attribute_byte_count = self.parse_u16()?;
        }

        Ok(Mesh::new(self.facets))
    }

    /// How many bytes are left in the buffer
    fn bytes_remaining(&self) -> usize {
        self.buf.len() - self.index
    }

    /// Skip the header. Returns `Err` if the header is missing (i.e. the file is smaller than 80 bytes)
    fn eat_header(&mut self) -> Result<(), Error> {
        // STL requires the header
        if self.bytes_remaining() < Self::HEADER_LENGTH {
            Err(Error::StlParse)
        } else {
            self.index += Self::HEADER_LENGTH;
            Ok(())
        }
    }

    /// Parse the next u16 from the buffer
    fn parse_u16(&mut self) -> Result<u16, Error> {
        const NUM_BYTES: usize = std::mem::size_of::<u16>();
        let bytes: [u8; NUM_BYTES] = self.buf[self.index..]
            .try_into()
            .map_err(|_| Error::StlParse)?;
        self.index += NUM_BYTES;

        Ok(u16::from_le_bytes(bytes))
    }

    /// Parse the next u32 from the buffer
    fn parse_u32(&mut self) -> Result<u32, Error> {
        const NUM_BYTES: usize = std::mem::size_of::<u32>();
        let bytes: [u8; NUM_BYTES] = self.buf[self.index..]
            .try_into()
            .map_err(|_| Error::StlParse)?;
        self.index += NUM_BYTES;

        Ok(u32::from_le_bytes(bytes))
    }

    /// Parse the next f32 from the buffer
    fn parse_f32(&mut self) -> Result<f32, Error> {
        const NUM_BYTES: usize = std::mem::size_of::<f32>();
        let bytes: [u8; NUM_BYTES] = self.buf[self.index..]
            .try_into()
            .map_err(|_| Error::StlParse)?;
        self.index += NUM_BYTES;

        Ok(f32::from_le_bytes(bytes))
    }

    /// Parse the next `Point` from the buffer
    fn parse_point(&mut self) -> Result<Point, Error> {
        Ok(Point::new(self.parse_f32()?, self.parse_f32()?, self.parse_f32()?))
    }

    /// Parse the next `Facet` from the buffer
    fn parse_facet(&mut self) -> Result<Facet, Error> {
        let normal = self.parse_point()?;
        Ok(Facet::new([self.parse_point()?, self.parse_point()?, self.parse_point()?], normal))
    }
}
