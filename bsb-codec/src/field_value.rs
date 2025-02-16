use std::fmt::Display;

use bsb_parser::Frame;
use serde::{Deserialize, Serialize};

use crate::{error::CodecError, field::Field, typed_value::TypedValue};

/// `FieldValue` contains information about the `Field` (via `field_id`) and the `TypedValue`.
/// Due to the construction, it is guaranteed that the field is supported by this crate.
/// It can be used to render a datapoint
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FieldValue {
    field_id: u32,
    typed_value: TypedValue,
}

impl FieldValue {
    /// Create a new `FieldValue` based on a `typed_value` and a `field_id` that is
    /// guaranteed to exist if it returns a Result
    pub fn new(field_id: u32, typed_value: TypedValue) -> Result<FieldValue, CodecError> {
        let field = Field::by_id(field_id)?;
        Ok(FieldValue {
            field_id: field.id(),
            typed_value,
        })
    }

    /// Convert a `Frame` to a `FieldValue` if that `Field` is known
    pub fn from_frame(frame: &Frame) -> Result<FieldValue, CodecError> {
        let field = Field::by_id(frame.field_id())?;
        let typed_value = TypedValue::decode(frame.payload(), field.datatype())?;
        Ok(FieldValue {
            field_id: frame.field_id(),
            typed_value,
        })
    }

    /// `path` to the datapoint (e.g. for MQTT)
    pub fn path(&self) -> &'static str {
        self.field().path()
    }

    /// Access `field_id`
    pub fn field_id(&self) -> u32 {
        self.field_id
    }

    /// Access `field`
    pub fn field(&self) -> &'static Field {
        let field =
            Field::by_id(self.field_id).expect("field is expected to exist due to construction");
        field
    }

    /// Access `typed_value`
    pub fn typed_value(&self) -> &TypedValue {
        &self.typed_value
    }

    /// Create a FieldValue from a string representation based on the datatype
    pub fn from_str(s: &str, field_id: u32) -> Result<FieldValue, CodecError> {
        let field = Field::by_id(field_id)?;
        let typed_value = TypedValue::from_str(s, field.datatype())?;
        Ok(FieldValue {
            field_id,
            typed_value,
        })
    }

    /// String representation of this value
    pub fn value_str(&self) -> String {
        self.typed_value.to_string()
    }

    /// Convert the payload to byte representation
    pub fn encode(&self) -> Vec<u8> {
        self.typed_value.encode()
    }

    /// Provide a default `FieldValue` for `Field`. The default is the Zero of this datatype
    pub fn default_for_field(field: &'static Field) -> FieldValue {
        FieldValue {
            field_id: field.id(),
            typed_value: TypedValue::default_for_datatype(field.datatype()),
        }
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field(), self.typed_value)
    }
}

#[cfg(test)]
mod tests {
    use bsb_parser::Frame;

    use crate::{
        datatypes::Datatype, error::CodecError, field::Field, typed_value::TypedValue, value::Value,
    };

    use super::FieldValue;

    fn create_test_field_value() -> FieldValue {
        FieldValue {
            field_id: 87890416,
            typed_value: TypedValue::new(Datatype::Float(10), Some(0), Value::Float(1.5)).unwrap(),
        }
    }

    #[test]
    fn test_field_value_from_frame() {
        let frame = Frame::new(66, 0, 7, 87890416, vec![0, 0, 15]);
        let testcase = FieldValue::from_frame(&frame).unwrap();
        let want = create_test_field_value();
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_from_str() {
        let testcase = FieldValue::from_str("1.5", 87890416).unwrap();
        let want = create_test_field_value();
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_access_path() {
        let testcase = create_test_field_value().path();
        let want = "system/water_pressure";
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_access_field() {
        let testcase = create_test_field_value().field();
        let want = Field::by_id(87890416).unwrap();
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_access_typed_value() {
        let test_field_value = create_test_field_value();
        let testcase = test_field_value.typed_value();
        let want = &TypedValue::new(Datatype::Float(10), Some(0), Value::Float(1.5)).unwrap();
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_to_value_str() {
        let testcase = create_test_field_value().value_str();
        let want = "1.5";
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_to_string() {
        let testcase = create_test_field_value().to_string();
        let want = "water_pressure: 1.5".to_string();
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_encode() {
        let testcase = create_test_field_value().encode();
        let want = vec![0, 0, 15];
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_default_for_field() {
        let field = Field::by_id(87890416).unwrap();
        let testcase = FieldValue::default_for_field(field);
        let want = FieldValue {
            field_id: field.id(),
            typed_value: TypedValue::new(Datatype::Float(10), Some(0), Value::Float(0.0)).unwrap(),
        };
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_from_frame_invalid() {
        let frame = Frame::new(66, 0, 7, 222103850, vec![0, 3]);
        let testcase = FieldValue::from_frame(&frame).expect_err("not an error");
        assert_eq!(testcase, CodecError::InvalidSetting);
    }
}
