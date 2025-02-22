pub use parser::ParseResult;
use strum_macros::FromRepr;

mod parser;
mod serializer;

/// BSB `SOF` (start of frame) that is used to start each frame
pub const SOF: u8 = 0xdc;

/// `Frame` contains all information that will be put on and read from the bus
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    destination_address: u8,
    source_address: u8,
    packet_type: u8,
    field_id: u32,
    payload: Vec<u8>,
}

impl Frame {
    /// Create a new Bsb `Frame`
    #[must_use]
    pub fn new(
        destination_address: u8,
        source_address: u8,
        packet_type: u8,
        field_id: u32,
        payload: Vec<u8>,
    ) -> Frame {
        Self {
            destination_address,
            source_address,
            packet_type,
            field_id,
            payload,
        }
    }

    /// Create a new Bsb `Frame` for a `Get` type frame
    #[must_use]
    pub fn new_get(destination_address: u8, source_address: u8, field_id: u32) -> Self {
        Self::new(
            destination_address,
            source_address,
            PacketType::Get as u8,
            field_id,
            vec![],
        )
    }

    /// Create a new Bsb `Frame` for a `Set` type frame
    #[must_use]
    pub fn new_set(
        destination_address: u8,
        source_address: u8,
        field_id: u32,
        payload: Vec<u8>,
    ) -> Self {
        Self::new(
            destination_address,
            source_address,
            PacketType::Set as u8,
            field_id,
            payload,
        )
    }

    /// Serialize the `Frame` into a `Vec<u8>`
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        serializer::frame_serializer(self)
    }

    /// Parse the `input` slice into `Ok(remaining_bytes, Frame)`, `Incomplete` or `Error`
    #[must_use]
    pub fn parse(input: &[u8]) -> ParseResult {
        parser::parser(input)
    }

    /// `destination_address` of this `Frame`
    #[must_use]
    pub fn destination_address(&self) -> u8 {
        self.destination_address
    }

    /// `source_address` of this `Frame`
    #[must_use]
    pub fn source_address(&self) -> u8 {
        self.source_address
    }

    /// `packet_type` of this `Frame`
    #[must_use]
    pub fn packet_type(&self) -> u8 {
        self.packet_type
    }

    /// `field_id` of this `Frame`
    #[must_use]
    pub fn field_id(&self) -> u32 {
        self.field_id
    }

    /// `payload` of this `Frame`
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// `PacketType` of the `Frame`
#[repr(u8)]
#[derive(FromRepr)]
pub enum PacketType {
    Unknown0,
    Unknown1,
    Info,
    Set,
    Ack,
    Nack,
    Get,
    Ret,
    Error,
}

#[cfg(test)]
mod tests {
    use crate::{parser::ParseResult, Frame};

    /// create a test frame for all tests
    fn create_frame() -> Frame {
        Frame::new(1, 2, 3, 4, [5].to_vec())
    }

    /// create a serialized version of a frame for all tests
    fn create_serialized() -> &'static [u8] {
        &[220, 130, 1, 12, 3, 0, 0, 0, 4, 5, 219, 42]
    }

    #[test]
    fn test_parse() {
        let testcase = create_serialized();
        if let ParseResult::Ok { rest, frame } = Frame::parse(&testcase) {
            assert!(rest.is_empty());
            assert_eq!(frame, create_frame());
        } else {
            assert!(false)
        };
    }

    #[test]
    fn test_serialize() {
        let testcase = create_frame();
        let want = create_serialized();
        assert_eq!(testcase.serialize(), want);
    }

    #[test]
    fn test_parse_two_correct_frames() {
        let testcase = vec![create_serialized().to_vec(), create_serialized().to_vec()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let want = create_frame();
        let ParseResult::Ok { rest, frame } = Frame::parse(&testcase) else {
            panic!("not a frame");
        };
        assert!(!rest.is_empty());
        assert_eq!(frame, want);
        let ParseResult::Ok { rest, frame } = Frame::parse(rest) else {
            panic!("not a frame");
        };
        assert!(rest.is_empty());
        assert_eq!(frame, want);
    }

    #[test]
    fn test_destination_address() {
        assert_eq!(create_frame().destination_address(), 1);
    }
    #[test]
    fn test_source_address() {
        assert_eq!(create_frame().source_address(), 2);
    }
    #[test]
    fn test_packet_type() {
        assert_eq!(create_frame().packet_type(), 3);
    }
    #[test]
    fn test_field_id() {
        assert_eq!(create_frame().field_id(), 4);
    }
    #[test]
    fn test_payload() {
        assert_eq!(create_frame().payload(), [5]);
    }
}
