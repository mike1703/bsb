use std::str::FromStr as _;

use chrono::NaiveDateTime;

use crate::{Datatype, Value};

/// a set of successfull testcases with (<datatype>, <encoded_bytes>, <flag>, <decoded_value>, <value_str>)
pub(crate) fn datatype_value_success_testcases(
) -> Vec<(Datatype, Vec<u8>, Option<u8>, Value, &'static str)> {
    vec![
        (
            Datatype::Setting(2),
            vec![0, 1],
            Some(0),
            Value::Setting(1),
            "1",
        ),
        (
            Datatype::Number,
            vec![0, 0, 15],
            Some(0),
            Value::Number(15),
            "15",
        ),
        (
            Datatype::Float(10),
            vec![0, 0, 15],
            Some(0),
            Value::Float(1.5),
            "1.5",
        ),
        (
            Datatype::Float(50),
            vec![0, 0, 35],
            Some(0),
            Value::Float(0.7),
            "0.7",
        ),
        (
            Datatype::Float(64),
            vec![0, 5, 192],
            Some(0),
            Value::Float(23.0),
            "23",
        ),
        (
            Datatype::DateTime,
            vec![0, 124, 11, 11, 1, 9, 36, 57, 0],
            Some(0),
            Value::DateTime(NaiveDateTime::from_str("2024-11-11T09:36:57").unwrap()),
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
