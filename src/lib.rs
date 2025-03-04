#![warn(clippy::pedantic)]

mod datatypes;
mod error;
mod field;
mod field_value;
mod frame;
#[cfg(test)]
mod testcases;
mod typed_value;
mod value;

// re-exports these datastructures as public API
pub use datatypes::Datatype;
pub use error::BsbError;
pub use field::Field;
pub use field_value::FieldValue;
pub use frame::parser::ParseErrorKind;
pub use frame::parser::ParseResult;
pub use frame::Frame;
pub use frame::PacketType;
pub use typed_value::TypedValue;
pub use value::Value;
