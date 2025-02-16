use bsb::{Datatype, FieldValue, Frame, PacketType, ParseResult, Value};

fn main() {
    let data: &[u8; 14] = &[
        0xDC, 0x80, 0x42, 0xE, 0x7, 0x5, 0x3D, 0x19, 0xF0, 0x0, 0x0, 0xF, 0x1D, 0x74,
    ];

    // create a new `Frame` with a manual created payload that is generated with bsb_codec
    let field_id = 0x053d19f0;
    let value = Value::Float {
        flag: 0,
        value: 1.5,
        factor: 10,
    };
    let field_value = FieldValue::new(field_id, value.clone()).unwrap();
    let frame = Frame::new(66, 0, PacketType::Ret as u8, field_id, field_value.encode());
    let encoded = frame.serialize();
    // the serialized form is identical to the above data
    assert_eq!(data.to_vec(), encoded);

    // create a `FieldValue` with the bsb Frame parser from the `data` byte stream
    if let ParseResult::Ok { rest: _, frame } = Frame::parse(data) {
        let decoded = FieldValue::from_frame(&frame).unwrap();
        assert_eq!(*decoded.value(), value);
    }

    // create a `FieldValue` directly from the payload without decoding a `Frame`
    let decoded = Value::decode(&[0, 0, 15], Datatype::Float(10)).unwrap();
    // this is identical to the manual created value above
    assert_eq!(decoded, value);
}
