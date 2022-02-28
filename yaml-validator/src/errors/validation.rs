use thiserror::Error;

use crate::breadcrumb::{Breadcrumb, BreadcrumbSegment, BreadcrumbSegmentVec};

use super::GenericError;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ValidationErrorKind<'a> {
    #[error("wrong type, expected {expected} got {actual}")]
    WrongType {
        expected: &'static str,
        actual: &'a str,
    },
    #[error("special requirements for field not met: {error}")]
    ValidationError { error: &'a str },
    #[error("field '{field}' missing")]
    FieldMissing { field: &'a str },
    #[error("field '{field}' is not specified in the schema")]
    ExtraField { field: &'a str },
    #[error("unknown type specified: {unknown_type}")]
    UnknownType { unknown_type: &'a str },
    #[error("multiple errors were encountered: {errors:?}")]
    Multiple { errors: Vec<ValidationError<'a>> },
    #[error("schema '{uri}' references was not found")]
    UnknownSchema { uri: &'a str },
}

impl<'a> ValidationErrorKind<'a> {
    pub fn with_path(self, path: BreadcrumbSegmentVec<'a>) -> ValidationError<'a> {
        ValidationError {
            kind: self,
            state: Breadcrumb::new(path),
        }
    }

    pub fn with_path_name(self, path: &'a str) -> ValidationError<'a> {
        let mut err: ValidationError = self.into();
        err.state.push(BreadcrumbSegment::Name(path));
        err
    }

    pub fn with_path_index(self, index: usize) -> ValidationError<'a> {
        let mut err: ValidationError = self.into();
        err.state.push(BreadcrumbSegment::Index(index));
        err
    }
}

impl<'a> From<ValidationErrorKind<'a>> for ValidationError<'a> {
    fn from(kind: ValidationErrorKind<'a>) -> ValidationError<'a> {
        ValidationError {
            kind,
            state: Breadcrumb::default(),
        }
    }
}

impl<'a> From<Vec<ValidationError<'a>>> for ValidationError<'a> {
    fn from(errors: Vec<ValidationError<'a>>) -> Self {
        ValidationErrorKind::Multiple { errors }.into()
    }
}

impl<'a> From<GenericError<'a>> for ValidationErrorKind<'a> {
    fn from(e: GenericError<'a>) -> Self {
        match e {
            GenericError::WrongType { expected, actual } => {
                ValidationErrorKind::WrongType { expected, actual }
            }
            GenericError::FieldMissing { field } => ValidationErrorKind::FieldMissing { field },
            GenericError::ExtraField { field } => ValidationErrorKind::ExtraField { field },
            GenericError::Multiple { errors } => ValidationErrorKind::Multiple {
                errors: errors
                    .into_iter()
                    .map(ValidationErrorKind::from)
                    .map(ValidationError::from)
                    .collect(),
            },
        }
    }
}

impl<'a> From<GenericError<'a>> for ValidationError<'a> {
    fn from(e: GenericError<'a>) -> Self {
        ValidationErrorKind::from(e).into()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationError<'a> {
    pub kind: ValidationErrorKind<'a>,
    pub state: Breadcrumb<'a>,
}

impl<'a> ValidationError<'a> {
    fn flatten(&self, fmt: &mut std::fmt::Formatter<'_>, root: String) -> std::fmt::Result {
        match &self.kind {
            ValidationErrorKind::Multiple { errors } => {
                for err in errors {
                    err.flatten(fmt, format!("{}{}", root, self.state))?;
                }
            }
            err => writeln!(fmt, "{}{}: {}", root, self.state, err)?,
        }

        Ok(())
    }

    pub fn add_path_name(path: &'a str) -> impl Fn(ValidationError<'a>) -> ValidationError<'a> {
        move |mut err: ValidationError<'a>| -> ValidationError<'a> {
            err.state.push(BreadcrumbSegment::Name(path));
            err
        }
    }

    pub fn add_path_index(index: usize) -> impl Fn(ValidationError<'a>) -> ValidationError<'a> {
        move |mut err: ValidationError<'a>| -> ValidationError<'a> {
            err.state.push(BreadcrumbSegment::Index(index));
            err
        }
    }
}

impl<'a> std::fmt::Display for ValidationError<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.flatten(fmt, "#".to_string())
    }
}
