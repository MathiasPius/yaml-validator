use thiserror::Error;
use yaml_rust::{Yaml, ScanError};

pub(crate) type ValidationResult<'a> =
    std::result::Result<(), StatefulResult<YamlValidationError<'a>>>;

#[derive(Debug)]
pub(crate) struct StatefulResult<E> {
    pub error: E,
    pub path: Vec<String>,
}

pub trait PathContext {
    fn prepend(self, segment: String) -> Self;
}

impl<T, E> PathContext for Result<T, StatefulResult<E>> {
    fn prepend(self, segment: String) -> Self {
        self.map_err(|mut state| {
            state.path.push(segment);
            state
        })
    }
}

impl<'a> std::fmt::Display for StatefulResult<YamlValidationError<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in self.path.iter().rev() {
            write!(f, "{}", segment)?;
        }
        write!(f, ": {}", self.error)
    }
}

impl<'a> Into<StatefulResult<YamlValidationError<'a>>> for YamlValidationError<'a> {
    fn into(self) -> StatefulResult<YamlValidationError<'a>> {
        StatefulResult {
            error: self,
            path: vec![],
        }
    }
}

impl Into<StatefulResult<YamlSchemaError>> for YamlSchemaError {
    fn into(self) -> StatefulResult<YamlSchemaError> {
        StatefulResult {
            error: self,
            path: vec![],
        }
    }
}

impl<'a> Into<StatefulResult<YamlValidationError<'a>>> for StringValidationError {
    fn into(self) -> StatefulResult<YamlValidationError<'a>> {
        StatefulResult {
            error: self.into(),
            path: vec![],
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum YamlSchemaError {
    #[error("error reading yaml: {0}")]
    YamlScanError(#[from] ScanError),
    #[error("schema parsing yaml: {0}")]
    SchemaParsingError(&'static str),
    #[error("exptected type '{0}', got '{0}'")]
    WrongType(&'static str, &'static str),
    #[error("attempting to parse object as '{1}', but field type is '{0}'")]
    TypeMismatch(&'static str, String),
    #[error("missing field '{0}'")]
    MissingField(&'static str),
    #[error("unknown type '{0}'")]
    UnknownType(String),
}

pub(crate) trait OptionalField<T> {
    fn into_optional(self) -> Result<Option<T>, StatefulResult<YamlSchemaError>>;
}

impl<T> OptionalField<T> for Result<T, StatefulResult<YamlSchemaError>> {
    fn into_optional(self) -> Result<Option<T>, StatefulResult<YamlSchemaError>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(e) => match e.error {
                YamlSchemaError::MissingField(_) => Ok(None),
                _ => Err(e),
            },
        }
    }
}

#[derive(Error, Debug)]
pub enum YamlValidationError<'a> {
    #[error("number validation error: {0}")]
    NumberValidationError(#[from] NumberValidationError),
    #[error("string validation error: {0}")]
    StringValidationError(#[from] StringValidationError),
    #[error("list validation error: {0}")]
    ListValidationError(#[from] ListValidationError),
    #[error("dictionary validation error: {0}")]
    DictionaryValidationError(#[from] DictionaryValidationError),
    #[error("object validation error: {0}")]
    ObjectValidationError(#[from] ObjectValidationError),
    #[error("wrong type, expected '{0}' got '{1:?}'")]
    WrongType(&'static str, &'a Yaml),
    #[error("missing field, '{0}' not found")]
    MissingField(&'a str),
    #[error("missing schema, '{0}' not found")]
    MissingSchema(&'a str),
    #[error("no context defined, but schema references other schema")]
    MissingContext,
}

#[derive(Error, Debug)]
pub enum NumberValidationError {}

#[derive(Error, Debug)]
pub enum StringValidationError {
    #[error("string too long, max is {0}, but string is {1}")]
    TooLong(usize, usize),
    #[error("string too short, min is {0}, but string is {1}")]
    TooShort(usize, usize),
}

#[derive(Error, Debug)]
pub enum ListValidationError {}

#[derive(Error, Debug)]
pub enum DictionaryValidationError {}

#[derive(Error, Debug)]
pub enum ObjectValidationError {}
