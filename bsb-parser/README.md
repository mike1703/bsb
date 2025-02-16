# bsb-parser

`bsb-parser` is a low-level parser for the BSB (Boiler System Bus) protocol.  
It provides a lightweight and efficient way to convert raw byte streams into structured BSB frames.

## Example Usage
```rust
use bsb_parser::{Frame, ParseResult};

fn main() {
    let data = &[
        0xDC, 0xC2, 0x00, 0x0B, 0x06, 0x3D, 0x05, 0x19, 0xF0, 0x24, 0x3E,
    ];
    if let ParseResult::Ok { rest, frame } = Frame::parse(data) {
        println!("Parsed frame: {:?}, unparsed: {:?}", frame, rest);
    }
}
```