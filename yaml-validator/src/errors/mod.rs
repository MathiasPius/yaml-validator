pub(crate) mod schema;
pub(crate) mod validation;

pub use schema::{SchemaError, SchemaErrorKind};
pub use validation::{ValidationError, ValidationErrorKind};

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum GenericError<'a> {
    #[error("wrong type, expected {expected} got {actual}")]
    WrongType {
        expected: &'static str,
        actual: &'a str,
    },
    #[error("field '{field}' missing")]
    FieldMissing { field: &'a str },
    #[error("field '{field}' is not specified in the schema")]
    ExtraField { field: &'a str },
    #[error("malformed field: {error}")]
    MalformedField { error: String },
    #[error("multiple errors were encountered: {errors:?}")]
    Multiple { errors: Vec<GenericError<'a>> },
}
