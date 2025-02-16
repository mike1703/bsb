use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum BsbError {
    #[error("invalid setting")]
    InvalidSetting,
    #[error("invalid schedule")]
    InvalidSchedule,
    #[error("invalid date time")]
    InvalidDateTime,
    #[error("invalid payload length")]
    InvalidPayloadLength,
    #[error("cannot parse FieldValue string")]
    InvalidFieldValue,
    #[error("no flag")]
    NoFlag,
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    ParseDateTimeError(#[from] chrono::ParseError),
    #[error("unsupported field")]
    UnsupportedField,
}
