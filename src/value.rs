use std::fmt::Display;

use chrono::{DateTime, Datelike as _, NaiveDate, NaiveDateTime, NaiveTime, Timelike as _};
use serde::{Deserialize, Serialize};

use crate::{BsbError, Datatype};

/// The Value enum is aligned with the Datatype enum
/// This type stores the actual values together with flags if necessary,
/// It is self sufficient to encode the value into a valid payload
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Value {
    /// Setting value based on u8 representation of the enum for this field
    Setting {
        flag: u8,
        setting: u8,
        max: u8,
    },
    /// A integer for e.g. error codes
    Number {
        flag: u8,
        value: u16,
    },
    /// Float numbers like pressure, slope, temperature
    Float {
        flag: u8,
        value: f32,
        factor: u8,
    },
    DateTime {
        flag: u8,
        datetime: chrono::NaiveDateTime,
    },
    // List of time ranges
    Schedule(Vec<(u8, u8, u8, u8)>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Setting { setting: v, .. } => write!(f, "{v}"),
            Value::Number { value: v, .. } => write!(f, "{v}"),
            Value::Float { value: v, .. } => write!(f, "{v}"),
            Value::DateTime { datetime: v, .. } => write!(f, "{}", v.format("%Y-%m-%dT%H:%M:%S")),
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
    /// Encode the `Value` into a `Vec<u8>` that can be used in a BSB protocol payload
    pub fn encode(&self) -> Vec<u8> {
        match &self {
            Value::Setting { flag, setting, .. } => {
                // this is the value for the payload
                vec![*flag, *setting]
            }
            Value::Number { flag, value } => {
                let mut r = (value).to_be_bytes().to_vec();
                r.insert(0, *flag);
                r
            }
            Value::Float {
                flag,
                value,
                factor,
            } => {
                let scaled_number = (value * f32::from(*factor)) as u16;
                let bytes = scaled_number.to_be_bytes();
                vec![*flag, bytes[0], bytes[1]]
            }
            Value::DateTime { flag, datetime } => {
                let value = datetime;
                vec![
                    *flag,
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

    /// Decode the BSB protocol `payload` with the specified `datatype` into a `Value`.
    /// Returns None if invalid settings are encountered
    pub fn decode(payload: &[u8], datatype: Datatype) -> Result<Value, BsbError> {
        let value = match datatype {
            Datatype::Setting(max) => {
                // use the second byte in the payload as the integer value for the enum
                let setting = *payload.get(1).ok_or(BsbError::InvalidPayloadLength)?;
                if setting > max {
                    // this seems to be an invalid setting, don't decode this
                    return Err(BsbError::InvalidSetting);
                }
                Value::Setting {
                    flag: *payload.get(0).ok_or(BsbError::NoFlag)?,
                    setting,
                    max,
                }
            }
            Datatype::Number => {
                if payload.len() < 3 {
                    return Err(BsbError::InvalidPayloadLength);
                };

                // unclear if this is unsigned
                Value::Number {
                    flag: *payload.get(0).ok_or(BsbError::NoFlag)?,
                    value: u16::from_be_bytes(payload[1..3].try_into().unwrap()),
                }
            }
            Datatype::Float(factor) => {
                if payload.len() < 3 {
                    return Err(BsbError::InvalidPayloadLength);
                }

                // signed 16bit integer with a division factor
                Value::Float {
                    flag: *payload.get(0).ok_or(BsbError::NoFlag)?,
                    value: i16::from_be_bytes(payload[1..3].try_into().ok().unwrap()) as f32
                        / factor as f32,
                    factor,
                }
            }
            Datatype::DateTime => {
                if payload.len() < 9 {
                    return Err(BsbError::InvalidPayloadLength);
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
                Value::DateTime {
                    flag: *payload.get(0).ok_or(BsbError::NoFlag)?,
                    datetime: NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(year, month, day)
                            .ok_or(BsbError::InvalidDateTime)?,
                        NaiveTime::from_hms_opt(hour, minute, second)
                            .ok_or(BsbError::InvalidDateTime)?,
                    ),
                }
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
                        return Err(BsbError::InvalidSchedule);
                    }
                    ranges.push((sh, sm, eh, em));
                }
                // if there is remaining data, the schedule was not provided in chunks of 4 bytes
                if !range.remainder().is_empty() {
                    return Err(BsbError::InvalidSchedule);
                }
                Value::Schedule(ranges)
            }
        };
        Ok(value)
    }

    // reverse of Display for Value
    pub fn from_str(s: &str, datatype: Datatype) -> Result<Value, BsbError> {
        match datatype {
            Datatype::Setting(max) => {
                let setting = s.parse::<u8>()?;
                if setting > max {
                    return Err(BsbError::InvalidSetting);
                }
                Ok(Value::Setting {
                    flag: 0,
                    setting,
                    max,
                })
            }
            Datatype::Number => {
                let value = s.parse::<u16>()?;
                Ok(Value::Number { flag: 0, value })
            }
            Datatype::Float(factor) => {
                let value = s.parse::<f32>()?;
                Ok(Value::Float {
                    flag: 0,
                    value,
                    factor,
                })
            }
            Datatype::DateTime => {
                let datetime = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")?;
                Ok(Value::DateTime { flag: 0, datetime })
            }
            Datatype::Schedule => {
                let mut ranges = Vec::new();
                // "<range>,<range>,<range>"
                for range in s.split(',') {
                    // "{sh}:{sm}-{eh}:{em}"
                    let (sh, rest) = range.split_once(':').ok_or(BsbError::InvalidSchedule)?;
                    let (sm, rest) = rest.split_once('-').ok_or(BsbError::InvalidSchedule)?;
                    let (eh, em) = rest.split_once(':').ok_or(BsbError::InvalidSchedule)?;
                    let sh = sh.parse::<u8>()?;
                    let sm = sm.parse::<u8>()?;
                    let eh = eh.parse::<u8>()?;
                    let em = em.parse::<u8>()?;
                    // validate correct hour and minute values
                    if sh > 24 || eh > 24 || sm > 59 || em > 59 {
                        return Err(BsbError::InvalidSchedule);
                    }
                    ranges.push((sh, sm, eh, em));
                }
                Ok(Value::Schedule(ranges))
            }
        }
    }

    /// Access the `flag` if available
    pub fn flag(&self) -> Option<u8> {
        match self {
            Value::Setting { flag, .. }
            | Value::Number { flag, .. }
            | Value::Float { flag, .. }
            | Value::DateTime { flag, .. } => Some(*flag),
            Value::Schedule(_) => None,
        }
    }

    /// Set the `flag` of the `Value` for all applicable types
    pub fn set_flag(&mut self, new_flag: u8) {
        match self {
            Value::Setting { flag, .. }
            | Value::Number { flag, .. }
            | Value::Float { flag, .. }
            | Value::DateTime { flag, .. } => *flag = new_flag,
            Value::Schedule(..) => {}
        }
    }

    /// Retrieve the datatype of this value
    pub fn datatype(&self) -> Datatype {
        match self {
            Value::Setting { max, .. } => Datatype::Setting(*max),
            Value::Number { .. } => Datatype::Number,
            Value::Float { factor, .. } => Datatype::Float(*factor),
            Value::DateTime { .. } => Datatype::DateTime,
            Value::Schedule(_) => Datatype::Schedule,
        }
    }

    /// Retrieve a default (Zero) `Value` for the specified `Datatype`
    pub fn default_for_datatype(datatype: Datatype) -> Value {
        match datatype {
            Datatype::Setting(max) => Value::Setting {
                flag: 0,
                setting: 0,
                max,
            },
            Datatype::Number => Value::Number { flag: 0, value: 0 },
            Datatype::Float(factor) => Value::Float {
                flag: 0,
                value: 0.0,
                factor,
            },
            Datatype::DateTime => Value::DateTime {
                flag: 0,
                datetime: DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            },
            Datatype::Schedule => Value::Schedule(vec![(0, 0, 0, 0)]),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use chrono::{DateTime, NaiveDateTime};

    use crate::{BsbError, Datatype, Value};

    /// a set of successfull testcases with (<datatype>, <encoded_bytes>, <flag>, <decoded_value>, <value_str>)
    fn datatype_value_success_testcases(
    ) -> Vec<(Datatype, Vec<u8>, Option<u8>, Value, &'static str)> {
        vec![
            (
                Datatype::Setting(2),
                vec![0, 1],
                Some(0),
                Value::Setting {
                    flag: 0,
                    setting: 1,
                    max: 2,
                },
                "1",
            ),
            (
                Datatype::Number,
                vec![0, 0, 15],
                Some(0),
                Value::Number { flag: 0, value: 15 },
                "15",
            ),
            (
                Datatype::Float(10),
                vec![0, 0, 15],
                Some(0),
                Value::Float {
                    flag: 0,
                    value: 1.5,
                    factor: 10,
                },
                "1.5",
            ),
            (
                Datatype::Float(50),
                vec![0, 0, 35],
                Some(0),
                Value::Float {
                    flag: 0,
                    value: 0.7,
                    factor: 50,
                },
                "0.7",
            ),
            (
                Datatype::Float(64),
                vec![0, 5, 192],
                Some(0),
                Value::Float {
                    flag: 0,
                    value: 23.0,
                    factor: 64,
                },
                "23",
            ),
            (
                Datatype::DateTime,
                vec![0, 124, 11, 11, 1, 9, 36, 57, 0],
                Some(0),
                Value::DateTime {
                    flag: 0,
                    datetime: NaiveDateTime::from_str("2024-11-11T09:36:57").unwrap(),
                },
                "2024-11-11T09:36:57",
            ),
            (
                Datatype::Schedule,
                vec![6, 50, 7, 10, 18, 30, 18, 50, 24 ^ 0x80, 0, 24, 0],
                None,
                Value::Schedule(vec![(6, 50, 7, 10), (18, 30, 18, 50)]),
                "6:50-7:10,18:30-18:50",
            ),
        ]
    }

    #[test]
    fn test_value_decode() {
        for (datatype, bytes, _flag, value, _display_str) in
            datatype_value_success_testcases().into_iter()
        {
            let testcase = Value::decode(&bytes, datatype).unwrap();
            let want = value;
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_encode() {
        for (_datatype, bytes, _flag, value, _display_str) in
            datatype_value_success_testcases().into_iter()
        {
            let testcase = value.encode();
            let want = bytes;
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_decode_encode_identical() {
        for (datatype, bytes, _flag, _value, _display_str) in
            datatype_value_success_testcases().into_iter()
        {
            let decoded = Value::decode(&bytes, datatype).unwrap();
            let testcase_encoded = decoded.encode();
            assert_eq!(testcase_encoded, bytes);
        }
    }

    #[test]
    fn test_value_encode_decode_identical() {
        for (datatype, _bytes, _flag, value, _display_str) in
            datatype_value_success_testcases().into_iter()
        {
            let want = value;
            let encoded = want.encode();
            let testcase_decoded = Value::decode(&encoded, datatype).unwrap();
            assert_eq!(testcase_decoded, want);
        }
    }

    #[test]
    fn test_value_to_string() {
        for (_datatype, _bytes, _flag, value, display_str) in
            datatype_value_success_testcases().into_iter()
        {
            let testcase = value.to_string();
            let want = display_str.to_string();
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_from_string() {
        for (datatype, _bytes, _flag, value, display_str) in
            datatype_value_success_testcases().into_iter()
        {
            let testcase = Value::from_str(display_str, datatype).unwrap();
            let want = value;
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_from_to_string_identical() {
        for (datatype, _bytes, _flag, _value, display_str) in
            datatype_value_success_testcases().into_iter()
        {
            let testcase = Value::from_str(display_str, datatype).unwrap().to_string();
            let want = display_str.to_string();
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_to_from_string_identical() {
        for (datatype, _bytes, _flag, value, _display_str) in
            datatype_value_success_testcases().into_iter()
        {
            let testcase = Value::from_str(&value.to_string(), datatype).unwrap();
            let want = value;
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_access_flag() {
        for (_datatype, _bytes, flag, value, _display_str) in
            datatype_value_success_testcases().into_iter()
        {
            let testcase = value.flag();
            let want = flag;
            assert_eq!(testcase, want);
        }
    }

    #[test]
    fn test_value_set_flag() {
        for (datatype, _bytes, _flag, mut value, _display_str) in
            datatype_value_success_testcases().into_iter()
        {
            value.set_flag(1);
            let testcase = value.flag();
            let want = Some(1);
            if datatype == Datatype::Schedule {
                // schedule does not have a flag
                assert_eq!(value.flag(), None);
            } else {
                assert_eq!(testcase, want);
            }
        }
    }

    #[test]
    fn test_value_from_string_errors() {
        // a set of error testcases for the value from string method (<datatype>, <string>, <error>)
        let from_string_error_testcases = vec![
            (Datatype::Setting(2), "3", BsbError::InvalidSetting),
            (
                Datatype::Schedule,
                "6:50-7:10,18:30-18:60",
                BsbError::InvalidSchedule,
            ),
            (
                Datatype::Schedule,
                "6:50-7:10,18:3018:50",
                BsbError::InvalidSchedule,
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
            Value::Setting {
                flag: 0,
                setting: 0,
                max: 2
            }
        );
        assert_eq!(
            Value::default_for_datatype(Datatype::Number),
            Value::Number { flag: 0, value: 0 }
        );
        assert_eq!(
            Value::default_for_datatype(Datatype::Float(10)),
            Value::Float {
                flag: 0,
                value: 0.0,
                factor: 10
            }
        );
        assert_eq!(
            Value::default_for_datatype(Datatype::DateTime),
            Value::DateTime {
                flag: 0,
                datetime: DateTime::from_timestamp(0, 0).unwrap().naive_utc()
            }
        );
        assert_eq!(
            Value::default_for_datatype(Datatype::Schedule),
            Value::Schedule(vec![(0, 0, 0, 0)])
        );
    }

    #[test]
    fn test_value_decode_errors() {
        // a set of error testcases to test the decoder (<datatype>, <encoded>, <error>)
        let error_testcases = vec![
            (Datatype::Setting(2), vec![0, 3], BsbError::InvalidSetting),
            (Datatype::Number, vec![0, 0], BsbError::InvalidPayloadLength),
            (
                Datatype::Float(10),
                vec![0, 0],
                BsbError::InvalidPayloadLength,
            ),
            (
                Datatype::DateTime,
                vec![0, 124, 11, 11, 1, 9, 36, 57],
                BsbError::InvalidPayloadLength,
            ),
            (
                Datatype::DateTime,
                vec![0, 124, 11, 11, 1, 25, 36, 57, 0],
                BsbError::InvalidDateTime,
            ),
            (
                Datatype::Schedule,
                vec![6, 50, 7, 10, 18, 30, 18],
                BsbError::InvalidSchedule,
            ),
            (
                Datatype::Schedule,
                vec![6, 50, 7, 10, 18, 30, 18, 60, 24 ^ 0x80, 0, 24, 0],
                BsbError::InvalidSchedule,
            ),
        ];
        for (datatype, bytes, error) in error_testcases.into_iter() {
            let testcase = Value::decode(&bytes, datatype).expect_err("not an error");
            assert_eq!(testcase, error);
        }
    }
}
