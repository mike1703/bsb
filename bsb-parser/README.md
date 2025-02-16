# bsb-parser

`bsb-parser` is a low-level parser for the BSB (Boiler System Bus) protocol.  
It provides a lightweight and efficient way to convert raw byte streams into structured BSB frames.

## Example Usage

```rust
use bsb_parser::{Frame, ParseResult};

fn main() {
    let data = &[
        0xDC, 0x80, 0x42, 0xE, 0x7, 0x5, 0x3D, 0x19, 0xF0, 0x0, 0x0, 0xF, 0x1D, 0x74,
    ];
    if let ParseResult::Ok { rest, frame } = Frame::parse(data) {
        println!("Parsed frame: {:?}, unparsed: {:?}", frame, rest);
    }
}
```

## Installation

Add `bsb-parser` to your `Cargo.toml`:

```toml
[dependencies]
bsb-parser = "0.1"
```

## Protocol

The BSB protocol is a simple, byte-oriented protocol used for communication with heating systems.
The protocol is based on frames that are sent over a serial connection.

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

The following packet types are defined:

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

The checksum is a CRC16 checksum of the message. The checksum is calculated over the bytes from the destination to the payload.

## Contributing

Contributions are welcome! Please feel free to open issues or submit pull requests.
