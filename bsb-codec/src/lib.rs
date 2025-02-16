mod datatypes;
mod error;
mod field;
mod field_value;
#[cfg(test)]
mod testcases;
mod typed_value;
mod value;

// re-exports these datastructures as public API
pub use datatypes::Datatype;
pub use error::CodecError;
pub use field::Field;
pub use field_value::FieldValue;
pub use typed_value::TypedValue;
pub use value::Value;
