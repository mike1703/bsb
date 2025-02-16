use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{BsbError, Field, Frame, NamedValue, Value};

/// `FieldValue` contains information about the `Field` (via `field_id`) and the `Value`.
/// Due to the construction, it is guaranteed that the field is supported by this crate.
/// It can be used to render a datapoint
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FieldValue {
    field_id: u32,
    value: Value,
}

impl FieldValue {
    /// Create a new `FieldValue` based on a `value` and a `field_id` that is
    /// guaranteed to exist if it returns a `FieldValue`
    pub fn new(field_id: u32, value: Value) -> Result<FieldValue, BsbError> {
        let field = Field::by_id(field_id).ok_or(BsbError::UnsupportedField)?;
        Ok(FieldValue {
            field_id: field.id(),
            value,
        })
    }

    /// Convert a `Frame` to a `FieldValue` if that `Field` is known
    pub fn from_frame(frame: &Frame) -> Result<FieldValue, BsbError> {
        let field = Field::by_id(frame.field_id()).ok_or(BsbError::UnsupportedField)?;
        let value = Value::decode(frame.payload(), field.datatype())?;
        Ok(FieldValue {
            field_id: frame.field_id(),
            value,
        })
    }

    /// Access `FieldValue.field().path` (e.g. for MQTT)
    pub fn path(&self) -> &'static str {
        self.field().path()
    }

    /// Access `FieldValue.field_id`
    pub fn field_id(&self) -> u32 {
        self.field_id
    }

    /// Access `FieldValue.field`
    pub fn field(&self) -> &'static Field {
        Field::by_id(self.field_id).expect("field is expected to exist due to construction")
    }

    /// Access `FieldValue.value`
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Access a mutable `FieldValue.value` reference
    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    /// Create a `FieldValue` from a string representation based on the datatype.
    /// This is the reverse of Display for `FieldValue` which prints "`<fieldname>: <value_str>`"
    pub fn from_str(s: &str, field_id: u32) -> Result<FieldValue, BsbError> {
        let (name_str, value_str) = s.split_once(":").ok_or(BsbError::InvalidFieldValue)?;
        let field = Field::by_name(name_str.trim()).ok_or(BsbError::UnsupportedField)?;
        let value = Value::from_str(value_str.trim(), field.datatype())?;
        Ok(FieldValue { field_id, value })
    }

    /// Create a `FieldValue` from a string representatino of the value.
    /// This is the reverse of `FieldValue.value_str()`
    pub fn from_value_str(s: &str, field_id: u32) -> Result<FieldValue, BsbError> {
        let field = Field::by_id(field_id).ok_or(BsbError::UnsupportedField)?;
        let value = Value::from_str(s, field.datatype())?;
        Ok(FieldValue { field_id, value })
    }

    /// String representation of `FieldValue.value`
    pub fn value_str(&self) -> String {
        self.value.to_string()
    }

    /// Convert the payload value to byte representation
    pub fn encode(&self) -> Vec<u8> {
        self.value.encode()
    }

    /// Provide a default `FieldValue` for `Field`. The default is the Zero of this datatype
    pub fn default_for_field(field: &'static Field) -> FieldValue {
        FieldValue {
            field_id: field.id(),
            value: Value::default_for_datatype(field.datatype()),
        }
    }

    /// Creates a `NamedValue` from the `FieldValue`
    pub fn to_named_value(&self) -> NamedValue {
        NamedValue::new(self.field().name(), self.value_str())
    }

    /// Create a `FieldValue` from the `NameValue`
    pub fn from_named_value(named_value: &NamedValue) -> Result<FieldValue, BsbError> {
        let field = Field::by_name(named_value.name()).ok_or(BsbError::UnsupportedField)?;
        let value = Value::from_str(&named_value.value(), field.datatype())?;
        Ok(FieldValue {
            field_id: field.id(),
            value,
        })
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field(), self.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{BsbError, Field, Frame, NamedValue, Value};

    use super::FieldValue;

    fn create_test_field_value() -> FieldValue {
        FieldValue {
            field_id: 87890416,
            value: Value::Float {
                flag: 0,
                value: 1.5,
                factor: 10,
            },
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
        let testcase = FieldValue::from_str("water_pressure: 1.5", 87890416).unwrap();
        let want = create_test_field_value();
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_invalid_field_value_from_str() {
        let testcase = FieldValue::from_str("invalid: 1.5", 87890416).expect_err("not an error");
        assert_eq!(testcase, BsbError::UnsupportedField);
        let testcase =
            FieldValue::from_str("water_pressure: invalid", 87890416).expect_err("not an error");
        matches!(testcase, BsbError::ParseFloatError(_));
        let testcase =
            FieldValue::from_str("water_pressure 1.5", 87890416).expect_err("not an error");
        assert_eq!(testcase, BsbError::InvalidFieldValue);
    }

    #[test]
    fn test_field_value_from_value_str() {
        let testcase = FieldValue::from_value_str("1.5", 87890416).unwrap();
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
    fn test_field_value_access_value() {
        let test_field_value = create_test_field_value();
        let testcase = test_field_value.value();
        let want = &Value::Float {
            flag: 0,
            value: 1.5,
            factor: 10,
        };
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_access_mut_flag() {
        let mut testcase = create_test_field_value();
        testcase.value_mut().set_flag(1);
        let want = Value::Float {
            flag: 1,
            value: 1.5,
            factor: 10,
        };
        assert_eq!(testcase.value, want);
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
            value: Value::Float {
                flag: 0,
                value: 0.0,
                factor: 10,
            },
        };
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_to_named_value() {
        let testcase = create_test_field_value().to_named_value();
        let want = NamedValue::new("water_pressure", "1.5".to_string());
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_from_named_value() {
        let named_value = NamedValue::new("water_pressure", "1.5".to_string());
        let testcase = FieldValue::from_named_value(&named_value).unwrap();
        let want = create_test_field_value();
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_field_value_from_named_value_invalid() {
        let named_value = NamedValue::new("invalid", "1.5".to_string());
        let testcase = FieldValue::from_named_value(&named_value).expect_err("not an error");
        assert_eq!(testcase, BsbError::UnsupportedField);
    }

    #[test]
    fn test_field_value_from_frame_invalid() {
        let frame = Frame::new(66, 0, 7, 222103850, vec![0, 3]);
        let testcase = FieldValue::from_frame(&frame).expect_err("not an error");
        assert_eq!(testcase, BsbError::InvalidSetting);
    }
}
