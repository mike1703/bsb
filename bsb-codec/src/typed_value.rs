use std::fmt::Display;

use chrono::{Datelike as _, NaiveDate, NaiveDateTime, NaiveTime, Timelike as _};
use serde::{Deserialize, Serialize};

use crate::{datatypes::Datatype, error::CodecError, value::Value};

/// a `Value` with its `Datatype`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedValue {
    datatype: Datatype,
    /// Unknown flag that's most of the time stored in the first byte
    flag: Option<u8>,
    value: Value,
}

impl TypedValue {
    /// Create a new `TypedValue`
    pub fn new(
        datatype: Datatype,
        flag: Option<u8>,
        value: Value,
    ) -> Result<TypedValue, CodecError> {
        // check if the value is valid for the datatype
        match (&value, &datatype) {
            (Value::Setting(v), Datatype::Setting(max)) => {
                if v > max {
                    return Err(CodecError::InvalidSetting);
                }
            }
            (Value::Float(_), Datatype::Float(_)) => {}
            (Value::Number(_), Datatype::Number) => {}
            (Value::DateTime(_), Datatype::DateTime) => {}
            (Value::Schedule(_), Datatype::Schedule) => {}
            _ => return Err(CodecError::InvalidDatatype),
        }

        Ok(TypedValue {
            datatype,
            flag,
            value,
        })
    }

    /// Access `TypedValue.value`
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Access `TypedValue.datatype`
    pub fn datatype(&self) -> &Datatype {
        &self.datatype
    }

    /// Access `TypedValue.flag`
    pub fn flag(&self) -> Option<u8> {
        self.flag
    }

    /// Create a TypedValue from string
    pub fn from_str(s: &str, datatype: Datatype) -> Result<TypedValue, CodecError> {
        let value = Value::from_str(s, datatype)?;
        Ok(TypedValue {
            datatype,
            flag: Some(0),
            value,
        })
    }

    /// Decode the BSB protocol `payload` with the specified `datatype` into a `TypedValue`.
    /// Returns None if invalid settings are encountered
    pub fn decode(payload: &[u8], datatype: Datatype) -> Result<TypedValue, CodecError> {
        // there is a flag for most datatypes in the first byte, but not for schedule
        let (value, flag) = match datatype {
            Datatype::Setting(max) => {
                // use the second byte in the payload as the integer value for the enum
                let enum_encoded_value = *payload.get(1).ok_or(CodecError::InvalidPayloadLength)?;
                if enum_encoded_value > max {
                    // this seems to be an invalid setting, don't decode this
                    return Err(CodecError::InvalidSetting);
                }
                (Value::Setting(enum_encoded_value), payload.get(0))
            }
            Datatype::Number => {
                if payload.len() < 3 {
                    return Err(CodecError::InvalidPayloadLength);
                };
                (
                    // unclear if this is unsigned
                    Value::Number(u16::from_be_bytes(payload[1..3].try_into().unwrap())),
                    payload.get(0),
                )
            }
            Datatype::Float(division_factor) => {
                if payload.len() < 3 {
                    return Err(CodecError::InvalidPayloadLength);
                }
                (
                    // signed 16bit integer with a division factor
                    Value::Float(
                        i16::from_be_bytes(payload[1..3].try_into().ok().unwrap()) as f32
                            / division_factor as f32,
                    ),
                    payload.get(0),
                )
            }
            Datatype::DateTime => {
                if payload.len() < 9 {
                    return Err(CodecError::InvalidPayloadLength);
                }
                // convert the payload bytes to the right datatypes
                let year = 1900 + payload[1] as i32;
                let month = payload[2] as u32;
                let day = payload[3] as u32;
                // day of week is currently not used - could be used as additional check
                let _dow = payload[4] as u32;
                let hour = payload[5] as u32;
                let minute = payload[6] as u32;
                let second = payload[7] as u32;
                let _unknown_flag = payload[8] as u32;
                (
                    Value::DateTime(NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(year, month, day)
                            .ok_or(CodecError::InvalidDateTime)?,
                        NaiveTime::from_hms_opt(hour, minute, second)
                            .ok_or(CodecError::InvalidDateTime)?,
                    )),
                    payload.get(0),
                )
            }
            Datatype::Schedule => {
                let mut ranges = Vec::new();
                let mut range = payload.chunks_exact(4);
                for chunk in &mut range {
                    let (sh, sm, eh, em) = (chunk[0], chunk[1], chunk[2], chunk[3]);
                    if sh & 0x80 != 0 {
                        break;
                    }
                    // validate correct hour and minute values
                    if sh > 24 || eh > 24 || sm > 59 || em > 59 {
                        return Err(CodecError::InvalidSchedule);
                    }
                    ranges.push((sh, sm, eh, em));
                }
                // if there is remaining data, the schedule was not provided in chunks of 4 bytes
                if !range.remainder().is_empty() {
                    return Err(CodecError::InvalidSchedule);
                }
                (Value::Schedule(ranges), None)
            }
        };
        Ok(TypedValue {
            datatype,
            flag: flag.copied(),
            value,
        })
    }

    /// Encode the `TypedValue` into a `Vec<u8>` that can be used in a BSB protocol payload
    pub fn encode(&self) -> Vec<u8> {
        match &self.value {
            Value::Setting(enum_value) => {
                // this is the value for the payload
                vec![
                    self.flag.expect("Setting needs to have a flag"),
                    *enum_value,
                ]
            }
            Value::Number(n) => {
                let mut r = (n).to_be_bytes().to_vec();
                r.insert(0, self.flag.expect("Number needs to have a flag"));
                r
            }
            Value::Float(n) => {
                let Datatype::Float(factor) = self.datatype else {
                    unimplemented!()
                };
                let scaled_number = (n * factor as f32) as u16;
                let bytes = scaled_number.to_be_bytes();
                vec![
                    self.flag.expect("Float needs to have a flag"),
                    bytes[0],
                    bytes[1],
                ]
            }
            Value::DateTime(t) => {
                let value = t;
                vec![
                    self.flag.expect("Datetime needs to have a flag"),
                    (value.year() - 1900).try_into().unwrap_or_default(),
                    value.month().try_into().unwrap(),
                    value.day().try_into().unwrap(),
                    value.weekday().number_from_monday().try_into().unwrap(),
                    value.hour().try_into().unwrap(),
                    value.minute().try_into().unwrap(),
                    value.second().try_into().unwrap(),
                    0, // some timezone flag? seen 1 already
                ]
            }
            Value::Schedule(items) => {
                let mut result = vec![];
                for (sh, sm, eh, em) in items {
                    result.extend_from_slice(&[*sh, *sm, *eh, *em]);
                }
                // terminate the schedule
                result.extend_from_slice(&[24 ^ 0x80, 0, 24, 0]);
                result
            }
        }
    }

    /// Create a default TypeValue for this datatype
    pub fn default_for_datatype(datatype: Datatype) -> TypedValue {
        TypedValue {
            datatype,
            flag: Some(0),
            value: Value::default_for_datatype(datatype),
        }
    }
}

impl Display for TypedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{testcases, CodecError, Datatype, TypedValue};

    #[test]
    fn test_typed_value_decode() {
        for (datatype, bytes, flag, value, _display_str) in
            testcases::success_testcases().into_iter()
        {
            let testcase = TypedValue::decode(&bytes, datatype).unwrap();
            let want = TypedValue::new(datatype, flag, value).unwrap();
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_typed_value_encode() {
        for (datatype, bytes, flag, value, _display_str) in
            testcases::success_testcases().into_iter()
        {
            let testcase = TypedValue::new(datatype, flag, value).unwrap().encode();
            let want = bytes;
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_typed_value_decode_encode_identical() {
        for (datatype, bytes, _flag, _value, _display_str) in
            testcases::success_testcases().into_iter()
        {
            let decoded = TypedValue::decode(&bytes, datatype).unwrap();
            let testcase_encoded = decoded.encode();
            assert_eq!(testcase_encoded, bytes);
        }
    }

    #[test]
    fn test_typed_value_to_string() {
        for (datatype, _bytes, flag, value, display_str) in
            testcases::success_testcases().into_iter()
        {
            let testcase = TypedValue::new(datatype, flag, value).unwrap().to_string();
            let want = display_str.to_string();
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_typed_value_decode_errors() {
        // a set of error testcases to test the decoder (<datatype>, <encoded>, <error>)
        let error_testcases = vec![
            (Datatype::Setting(2), vec![0, 3], CodecError::InvalidSetting),
            (
                Datatype::Number,
                vec![0, 0],
                CodecError::InvalidPayloadLength,
            ),
            (
                Datatype::Float(10),
                vec![0, 0],
                CodecError::InvalidPayloadLength,
            ),
            (
                Datatype::DateTime,
                vec![0, 124, 11, 11, 1, 9, 36, 57],
                CodecError::InvalidPayloadLength,
            ),
            (
                Datatype::DateTime,
                vec![0, 124, 11, 11, 1, 25, 36, 57, 0],
                CodecError::InvalidDateTime,
            ),
            (
                Datatype::Schedule,
                vec![6, 50, 7, 10, 18, 30, 18],
                CodecError::InvalidSchedule,
            ),
            (
                Datatype::Schedule,
                vec![6, 50, 7, 10, 18, 30, 18, 60, 24 ^ 0x80, 0, 24, 0],
                CodecError::InvalidSchedule,
            ),
        ];
        for (datatype, bytes, error) in error_testcases.into_iter() {
            let testcase = TypedValue::decode(&bytes, datatype).expect_err("not an error");
            assert_eq!(testcase, error);
        }
    }

    #[test]
    fn test_typed_value_encode_decode_identical() {
        for (datatype, _bytes, flag, value, _display_str) in
            testcases::success_testcases().into_iter()
        {
            let want = TypedValue::new(datatype, flag, value).unwrap();
            let encoded = want.encode();
            let testcase_decoded = TypedValue::decode(&encoded, datatype).unwrap();
            assert_eq!(testcase_decoded, want);
        }
    }

    #[test]
    fn test_typed_value_invalid_datatype_value_combination() {
        let testcase = TypedValue::new(Datatype::Number, Some(0), crate::Value::Float(1.0))
            .expect_err("no error");
        assert_eq!(testcase, CodecError::InvalidDatatype);
    }
}
