use cookie_factory::{
    bytes::{be_u16, be_u32, be_u8},
    combinator::slice,
    gen,
    sequence::tuple,
};

use super::{Frame, SOF};

pub struct FrameSerializer {}

impl FrameSerializer {
    /// Serialize the `Frame` into a `Vec<u8>`
    #[must_use]
    pub fn serialize(frame: &Frame) -> Vec<u8> {
        let header_length = frame.payload.len() + 4 + 4 + 2 + 1;
        // prepare buffer with correct length
        let mut buffer = vec![0; header_length];
        // generate the message without checksum
        let (_, pos) = gen(
            tuple((
                be_u8(SOF),
                be_u8(frame.source_address ^ 0x80),
                be_u8(frame.destination_address),
                be_u8(header_length.try_into().unwrap()),
                be_u8(frame.packet_type),
                be_u32(if frame.packet_type == 3 || frame.packet_type == 6 {
                    // for sets (3) and gets (6) these id bytes are swapped
                    (frame.field_id & 0x0000_ffff)
                        | ((frame.field_id >> 8) & 0x00ff_0000)
                        | ((frame.field_id << 8) & 0xff00_0000)
                } else {
                    frame.field_id
                }),
                slice(frame.payload.clone()),
            )),
            buffer.as_mut_slice(),
        )
        .unwrap();
        let pos = usize::try_from(pos).expect("pos is too big for usize");
        // calculate the checksum for the already serialized message
        let crc = crc16::State::<crc16::XMODEM>::calculate(&buffer[0..pos]);
        // and append it
        let (_, _) = gen(be_u16(crc), &mut buffer[pos..]).unwrap();

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::{Frame, FrameSerializer};

    #[test]
    fn test_frame_serialize() {
        let frame = Frame::new(66, 0, 7, 87890416, vec![0, 0, 15]);
        let testcase = FrameSerializer::serialize(&frame);
        let want = vec![220, 128, 66, 14, 7, 5, 61, 25, 240, 0, 0, 15, 29, 116];
        assert_eq!(want, testcase);
    }

    #[test]
    fn test_frame_serialize_get_request() {
        let frame = Frame::new_get(0, 66, 87890416);
        let testcase = FrameSerializer::serialize(&frame);
        let want = vec![220, 194, 0, 11, 6, 61, 5, 25, 240, 36, 62];
        assert_eq!(want, testcase);
    }

    #[test]
    fn test_frame_serialize_set_request() {
        let frame = Frame::new_set(0, 66, 87884342, vec![1, 0]);
        let testcase = FrameSerializer::serialize(&frame);
        let want = vec![220, 194, 0, 13, 3, 61, 5, 2, 54, 1, 0, 70, 13];
        assert_eq!(want, testcase);
    }
}
