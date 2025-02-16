use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::FieldValue;

/// `NamedValue` is optimized to contain all information necessary
/// for display purposes but can recover the original representation
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NamedValue {
    name: &'static str,
    value: String,
}

impl NamedValue {
    /// Create a new `NamedValue`
    pub fn new(name: &'static str, value: String) -> NamedValue {
        NamedValue { name, value }
    }

    /// Access `NamedValue.name`
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Access `NamedValue.value`
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Create a `FieldValue` from the `NamedValue`
    pub fn from_field_value(field_value: &FieldValue) -> NamedValue {
        field_value.to_named_value()
    }
}

impl Display for NamedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::NamedValue;

    fn create_test_named_value() -> NamedValue {
        NamedValue::new("test", "1.5".to_string())
    }

    #[test]
    fn test_named_value_to_string() {
        let testcase = create_test_named_value().to_string();
        let want = "test: 1.5".to_string();
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_named_value_access_name() {
        let testcase = create_test_named_value().name();
        let want = "test";
        assert_eq!(testcase, want);
    }

    #[test]
    fn test_named_value_access_value() {
        let named_value = create_test_named_value();
        let testcase = named_value.value();
        let want = "1.5";
        assert_eq!(testcase, want);
    }
}
