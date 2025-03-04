use thiserror::Error;

use crate::{field::FieldError, typed_value::TypedValueError, value::ValueError};

/// The common error type used in the bsb crate
#[derive(Debug, Error, PartialEq)]
pub enum BsbError {
    #[error(transparent)]
    Field(#[from] FieldError),
    #[error(transparent)]
    ValueError(#[from] ValueError),
    #[error(transparent)]
    TypedValueError(#[from] TypedValueError),
}
