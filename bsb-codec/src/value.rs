use std::fmt::Display;

use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::{datatypes::Datatype, error::CodecError};

/// The Value enum is aligned with the Datatype enum
/// This type stores the actual values
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
    /// Setting value based on u8 representation of the enum for this field
    Setting(u8),
    /// A integer for e.g. error codes
    Number(u16),
    /// Float numbers like pressure, slope, temperature
    Float(f32),
    DateTime(chrono::NaiveDateTime),
    // List of time ranges
    Schedule(Vec<(u8, u8, u8, u8)>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Setting(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::DateTime(v) => write!(f, "{}", v.format("%Y-%m-%dT%H:%M:%S")),
            Value::Schedule(v) => write!(
                f,
                "{}",
                v.iter()
                    .map(|(sh, sm, eh, em)| format!("{sh}:{sm}-{eh}:{em}"))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

impl Value {
    // reverse of Display for Value
    pub fn from_str(s: &str, datatype: Datatype) -> Result<Value, CodecError> {
        match datatype {
            Datatype::Setting(max) => {
                let v = s.parse::<u8>()?;
                if v > max {
                    return Err(CodecError::InvalidSetting);
                }
                Ok(Value::Setting(v))
            }
            Datatype::Number => {
                let v = s.parse::<u16>()?;
                Ok(Value::Number(v))
            }
            Datatype::Float(_factor) => {
                let v = s.parse::<f32>()?;
                Ok(Value::Float(v))
            }
            Datatype::DateTime => {
                let v = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")?;
                Ok(Value::DateTime(v))
            }
            Datatype::Schedule => {
                let mut ranges = Vec::new();
                // "<range>,<range>,<range>"
                for range in s.split(',') {
                    // "{sh}:{sm}-{eh}:{em}"
                    let (sh, rest) = range.split_once(':').ok_or(CodecError::InvalidSchedule)?;
                    let (sm, rest) = rest.split_once('-').ok_or(CodecError::InvalidSchedule)?;
                    let (eh, em) = rest.split_once(':').ok_or(CodecError::InvalidSchedule)?;
                    let sh = sh.parse::<u8>()?;
                    let sm = sm.parse::<u8>()?;
                    let eh = eh.parse::<u8>()?;
                    let em = em.parse::<u8>()?;
                    // validate correct hour and minute values
                    if sh > 24 || eh > 24 || sm > 59 || em > 59 {
                        return Err(CodecError::InvalidSchedule);
                    }
                    ranges.push((sh, sm, eh, em));
                }
                Ok(Value::Schedule(ranges))
            }
        }
    }

    /// retrieve a default (Zero) `Value` for the specified `Datatype`
    pub fn default_for_datatype(datatype: Datatype) -> Value {
        match datatype {
            Datatype::Setting(_) => Value::Setting(0),
            Datatype::Number => Value::Number(0),
            Datatype::Float(_) => Value::Float(0.0),
            Datatype::DateTime => {
                Value::DateTime(DateTime::from_timestamp(0, 0).unwrap().naive_utc())
            }
            Datatype::Schedule => Value::Schedule(vec![(0, 0, 0, 0)]),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::{testcases, CodecError, Datatype, Value};

    #[test]
    fn test_value_from_string() {
        for (datatype, _bytes, _flag, value, display_str) in
            testcases::success_testcases().into_iter()
        {
            let testcase = Value::from_str(display_str, datatype).unwrap();
            let want = value;
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_from_to_string_identical() {
        for (datatype, _bytes, _flag, _value, display_str) in
            testcases::success_testcases().into_iter()
        {
            let testcase = Value::from_str(display_str, datatype).unwrap().to_string();
            let want = display_str.to_string();
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_to_from_string_identical() {
        for (datatype, _bytes, _flag, value, _display_str) in
            testcases::success_testcases().into_iter()
        {
            let testcase = Value::from_str(&value.to_string(), datatype).unwrap();
            let want = value;
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_from_string_errors() {
        // a set of error testcases for the value from string method (<datatype>, <string>, <error>)
        let from_string_error_testcases = vec![
            (Datatype::Setting(2), "3", CodecError::InvalidSetting),
            (
                Datatype::Schedule,
                "6:50-7:10,18:30-18:60",
                CodecError::InvalidSchedule,
            ),
            (
                Datatype::Schedule,
                "6:50-7:10,18:3018:50",
                CodecError::InvalidSchedule,
            ),
        ];
        for (datatype, string, error) in from_string_error_testcases.into_iter() {
            let testcase = Value::from_str(string, datatype).expect_err("not an error");
            assert_eq!(testcase, error);
        }
    }

    #[test]
    fn test_value_default_for_datatype() {
        assert_eq!(
            Value::default_for_datatype(Datatype::Setting(2)),
            Value::Setting(0)
        );
        assert_eq!(
            Value::default_for_datatype(Datatype::Number),
            Value::Number(0)
        );
        assert_eq!(
            Value::default_for_datatype(Datatype::Float(10)),
            Value::Float(0.0)
        );
        assert_eq!(
            Value::default_for_datatype(Datatype::DateTime),
            Value::DateTime(DateTime::from_timestamp(0, 0).unwrap().naive_utc())
        );
        assert_eq!(
            Value::default_for_datatype(Datatype::Schedule),
            Value::Schedule(vec![(0, 0, 0, 0)])
        );
    }
}
