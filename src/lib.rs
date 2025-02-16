#![warn(clippy::pedantic)]

mod datatypes;
mod error;
mod field;
mod field_value;
mod frame;
mod named_value;
mod value;

// re-export these datastructures as public API
pub use datatypes::Datatype;
pub use error::BsbError;
pub use field::Field;
pub use field_value::FieldValue;
pub use frame::parser::ParseErrorKind;
pub use frame::parser::ParseResult;
pub use frame::Frame;
pub use frame::PacketType;
pub use named_value::NamedValue;
pub use value::Value;
