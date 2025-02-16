use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CodecError {
    #[error("invalid date time")]
    InvalidDateTime,
    #[error("invalid payload length")]
    InvalidPayloadLength,
    #[error("invalid schedule")]
    InvalidSchedule,
    #[error("invalid setting")]
    InvalidSetting,
    #[error("invalid value datatype")]
    InvalidDatatype,
    #[error("unknown field")]
    UnknownField,
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error(transparent)]
    ParseDateTimeError(#[from] chrono::ParseError),
}
