# bsb

`bsb` is a parser for the BSB (Boiler System Bus) wire protocol and decodes a variety of datatypes stored in the payload.

## Example Usage

```rust
use bsb::{Frame, ParseResult};

fn main() {
    let data = &[
        0xDC, 0x80, 0x42, 0xE, 0x7, 0x5, 0x3D, 0x19, 0xF0, 0x0, 0x0, 0xF, 0x1D, 0x74,
    ];
    if let ParseResult::Ok { rest, frame } = Frame::parse(data) {
        dbg!(rest);
        // rest = []
        dbg!(frame);
        // frame = Frame {
        //     destination_address: 66,
        //     source_address: 0,
        //     packet_type: 7,
        //     field_id: 87890416,
        //     payload: [
        //         0,
        //         0,
        //         15,
        //     ],
        // }
        let decoded = FieldValue::from_frame(&frame).unwrap();
        dbg!(decoded.to_string());
        // decoded.to_string() = "water_pressure: 1.5"
        dbg!(decoded);
        // decoded = FieldValue {
        //     field_id: 87890416,
        //     value: Float {
        //         flag: 0,
        //         value: 1.5,
        //         factor: 10,
        //     },
        // }
    }
}
```

## Installation

Add `bsb` to your `Cargo.toml`:

```toml
[dependencies]
bsb = "0.1"
```

## Protocol

The BSB protocol is a simple, byte-oriented protocol used for communication with heating systems.
The protocol is based on frames that are sent over a serial connection and this code is mostly based on research by listening to the bus while interacting with the official controller.

### Frame format

A BSB frame consists of the following parts:

- **Start byte** (1 byte): `0xDC`
- **Source** (1 byte): The source address of the message
- **Destination** (1 byte): The destination address of the message
- **Length** (1 byte): The length of the entire message in bytes (including start byte)
- **Packet type** (1 byte): The type of the message
- **Field ID** (4 bytes): The ID of the field that is being read or written
- **Payload** (0-n bytes): The data of the message
- **Checksum** (2 bytes): A CRC16 checksum of the message

### Packet types

The following packet types are defined (as far as known):

- Info (0x02),
- Set (0x03),
- Ack (0x04),
- Nack (0x05),
- Get (0x06),
- Ret (0x07),
- Error (0x08),

### Field ID

The field ID is a 4-byte value that identifies the field that is being read or written.
The field ID is used to determine the data type of the payload.

### Payload

The payload is a variable-length byte sequence that contains the data of the message.
The data type of the payload is determined by the field ID.

### Checksum

The checksum is a CRC16 checksum of the message. The checksum is calculated from all bytes of the header and the payload (excluding the SOF and checksum bytes).

## Payload

The payload contains the actual data and needs to be decoded.

### Payload data type

There is no information in the payload itself to determine its data type. The data type can only be determined by looking at the field id that is also stored in the BSB `Frame`. E.g. the field `0x313d052f` stands for the warm water temperature and has a temperature type (`Float(64)`). Knowing this datatype the payload can be decoded from the payload byte stream.

### Supported data types

- `Setting` - an integer that represents a specific setting in the specific fields context
- `Number` - an integer value for e.g. error codes
- `Float` - a signed f32 value with a division factor like 10 (pressure), 50 (slopes) or 64 (temperature)
- `DateTime` - a date/time format for e.g. the time
- `Schedule` - range of date time, e.g. warm water schedule

### data formats

The payload supports a variety of different data types and most of them are prefixed with a `flag` that is used as a `set` (if 1) flag. It might have other unknown purposes.

#### Float + Number

Floats and Number values are big-endian encoded with 3 bytes in the payload `[<flag?>,<msb>,<lsb>]`like `[0,0,15]` for the integer 15. Depending on the division factor of the datatype (e.g 10 for Pressure) it determines the resolution of the integer value. In this example the pressure value is `15 / 10 = 1.5`. A number is directly used as is.

#### DateTime

A datetime is decoded to the Rust `chrono::NaiveDateTime` is encoded as `[<flag?>, <year>-1900, <month>, <day>, <day_of_week(mon=1,sun=7)>, <hour>, <minute>, <second>, <timezone?>]`

#### Schedule

A schedule is defined as a range of times (max 3 ranges) with minute resolution e.g as `[<sh1>, <sm1>, <eh1>, <em1>, <sh2>, <sm2>, <eh2>, <em1>, … repeating two times]`. The last valid range is marked with the `0x80` bit set in the starting hour byte. It does not seem to have a flag byte.

#### Enums

Enum values are represented with a 2 byte payload `[<flag>, <enum_value>]` where the enum value is provided as an integer (mapping to strings describing this value is coming in later versions of the crate). The `flag` defines if this is a returned value or if this is set.

### Supported fields

Currently there is only a limited amount of fields supported by this crate. The fields are defined in [bsb-fields.csv](bsb-fields.csv) and converted into a static rust map with [build.rs](build.rs)
To decode a new field a new line has to be added to that csv and the crate needs to be rebuilt. Beside the datatype and a name that can be printed, there is a "path" style id that can be used e.g. as MQTT topic.

## Contributing

Contributions are welcome! Please feel free to open issues or submit pull requests.
This can include simple additional fields and datatypes but also changes to the overall code structure.
