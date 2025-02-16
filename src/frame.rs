use serde::Serialize;
use strum::FromRepr;

use crate::FieldValue;
use parser::{FrameParser, ParseResult};
use serializer::FrameSerializer;

pub(crate) mod parser;
pub(crate) mod serializer;

/// BSB `SOF` (start of frame) that is used to start each frame
pub const SOF: u8 = 0xdc;

/// `Frame` contains all information that will be put on and read from the bus
#[derive(Clone, Debug, PartialEq, Serialize)]
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
        Frame {
            destination_address,
            source_address,
            packet_type,
            field_id,
            payload,
        }
    }

    /// Create a new Bsb `Frame` for a `Get` type frame
    #[must_use]
    pub fn new_get(destination_address: u8, source_address: u8, field_id: u32) -> Frame {
        Frame::new(
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
    ) -> Frame {
        Frame::new(
            destination_address,
            source_address,
            PacketType::Set as u8,
            field_id,
            payload,
        )
    }

    /// Parse the `input` slice into `Ok(remaining_bytes, Frame)`, `Incomplete` or `Error`
    #[must_use]
    pub fn parse(input: &[u8]) -> ParseResult<'_> {
        FrameParser::parse(input)
    }

    /// Serialize the `Frame` into a `Vec<u8>`
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        FrameSerializer::serialize(self)
    }

    /// Access `Frame.destination_address`
    #[must_use]
    pub fn destination_address(&self) -> u8 {
        self.destination_address
    }

    /// Access `Frame.source_address`
    #[must_use]
    pub fn source_address(&self) -> u8 {
        self.source_address
    }

    /// Access `Frame.packet_type`
    #[must_use]
    pub fn packet_type(&self) -> u8 {
        self.packet_type
    }

    /// Access `field_id`
    #[must_use]
    pub fn field_id(&self) -> u32 {
        self.field_id
    }

    /// Access `payload`
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Decode the `payload` if the field is known
    pub fn try_decode(&self) -> Option<FieldValue> {
        FieldValue::from_frame(self).ok()
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
    use super::{parser::ParseResult, Frame};

    /// Create a test frame for all tests
    fn create_frame() -> Frame {
        Frame::new(1, 2, 3, 4, [5].to_vec())
    }

    /// Create a serialized version of a frame for all tests
    fn create_serialized() -> &'static [u8] {
        &[220, 2 ^ 0x80, 1, 12, 3, 0, 0, 0, 4, 5, 219, 42]
    }

    #[test]
    fn test_parse() {
        let testcase = create_serialized();
        let ParseResult::Ok { rest, frame } = Frame::parse(&testcase) else {
            panic!("not a frame")
        };
        assert!(rest.is_empty());
        assert_eq!(frame, create_frame());
    }

    #[test]
    fn test_serialize() {
        let testcase = create_frame();
        let want = create_serialized();
        assert_eq!(testcase.serialize(), want);
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

    #[test]
    fn test_decode() {
        let frame = Frame::new(66, 0, 7, 87890416, vec![0, 0, 15]);
        let testcase = frame.try_decode().unwrap();
        assert_eq!(testcase.value_str(), "1.5");
    }
}
