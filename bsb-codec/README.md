# bsb-codec

`bsb-codec` is a Rust library for decoding and encoding messages in the BSB (Boiler System Bus) protocol.  
It works alongside [`bsb-frame`](https://crates.io/crates/bsb-frame) to process BSB frames and extract meaningful data.

## Features

- **Decodes BSB frames** into structured message representations
- **Encodes structured messages** back into BSB frames
- **Embedded field DB** enables decoding of some fields
- **Works seamlessly with [`bsb-frame`](https://crates.io/crates/bsb-frame)** for raw frame parsing

## Installation

Add `bsb-codec` to your `Cargo.toml`:

```toml
[dependencies]
bsb-codec = "0.1"
```

## Example Usage

```rust
use bsb_codec::codec::FieldValue;
use bsb_parser::{Frame, ParseResult};

fn main() {
    // input stream
    let data: &[u8; 14] = &[
        0xDC, 0x80, 0x42, 0xE, 0x7, 0x5, 0x3D, 0x19, 0xF0, 0x0, 0x0, 0xF, 0x1D, 0x74,
    ];
    // create a `FieldValue` with bsb-parser from the `data` byte stream
    if let ParseResult::Ok { rest: _, frame } = Frame::parse(data) {
        let decoded = FieldValue::from_frame(&frame).unwrap();
        dbg!(decoded);
        // decoded = FieldValue {
        //     field: Field {
        //         name: "water_pressure",
        //         prognr: 8704,
        //         datatype: Float(
        //             10,
        //         ),
        //         path: "system/water_pressure",
        //     },
        //     value: TypedValue {
        //         datatype: Float(
        //             10,
        //         ),
        //         flag: Some(
        //             0,
        //         ),
        //         value: Float(
        //             1.5,
        //         ),
        //     },
        // }
    }
}
```

## Protocol

`bsb-codec` has a dependency on `bsb-frame` which parses the wire format into a frame. This crate does not care about the outer frame, destinations, frame types or checksums. It is soley for decoding the payload of the frame.

There is no information in the payload itself to determine its data type. The data type can only be determined by looking at the field id that is also stored in the BSB `Frame`. E.g. the field `0x313d052f` stands for the warm water temperature and has a `Temperature` type (`Float(64)`). Knowing this datatype the payload can be decoded from the payload byte stream.

### Supported data types

- `Setting` - an integer that represents a specific setting in the specific fields context
- `Number` - an integer value for e.g. error codes
- `Float` - a signed f32 value with a division factor like 10 (pressure), 50 (slopes) or 64 (temperature)
- `DateTime` - a date/time format for e.g. the time
- `Schedule` - range of date time, e.g. warm water schedule

### data formats

#### Float + Number

Floats and Number values are big-endian encoded with 3 bytes in the payload `[<flag?>,<msb>,<lsb>]`like `[0,0,15]` for the integer 15. Depending on the division factor of the datatype (e.g 10 for Pressure) it determines the resolution of the integer value. In this example the pressure value is `0x0015 / 10 = 1.5`. A number is directly used as is.

#### DateTime

A datetime is decoded to the Rust `chrono::NaiveDateTime` is encoded as `[<flag?>, <year>-1900, <month>, <day>, <day_of_week(mon=1,sun=7)>, <hour>, <minute>, <second>, <timezone?>]`

#### Schedule

A schedule is defined as a range of times (max 3 ranges) with minute resolution e.g as `[<sh1>, <sm1>, <eh1>, <em1>, <sh2>, <sm2>, <eh2>, <em1>, … repeating two times]`. The last valid range is marked with the `0x80` bit set.

#### Enums

Enum values are represented with a 2 byte payload `[<flag>, <enum_value>]` where the enum value is provided as an integer (mapping to strings describing this value is coming in later versions of the crate). The `flag` defines if this is a returned value or if this is set.

### Supported fields

Currently there is only a limited amount of fields supported by this crate. The fields are defined in [bsb-fields.csv](bsb-fields.csv) and converted into a static rust map with [build.rs](build.rs)
To decode a new field a new line has to be added to that csv and the crate needs to be rebuilt. Beside the datatype and a name that can be printed, there is a "path" style id that can be used e.g. as MQTT topic.

## Contributing

Contributions are welcome! Please feel free to open issues or submit pull requests.
