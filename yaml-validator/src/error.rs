use thiserror::Error;
use yaml_rust::{ScanError, Yaml};

pub(crate) type ValidationResult<'a> =
    std::result::Result<(), StatefulError<YamlValidationError<'a>>>;

#[derive(Debug)]
pub(crate) struct StatefulError<E> {
    pub error: E,
    pub path: Vec<String>,
}

pub trait PathContext {
    fn prepend(self, segment: String) -> Self;
}

impl<T, E> PathContext for Result<T, StatefulError<E>> {
    fn prepend(self, segment: String) -> Self {
        self.map_err(|mut state| {
            state.path.push(segment);
            state
        })
    }
}

impl<'a> std::fmt::Display for StatefulError<YamlValidationError<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in self.path.iter().rev() {
            write!(f, "{}", segment)?;
        }
        write!(f, ": {}", self.error)
    }
}

impl<'a> Into<StatefulError<YamlValidationError<'a>>> for YamlValidationError<'a> {
    fn into(self) -> StatefulError<YamlValidationError<'a>> {
        StatefulError {
            error: self,
            path: vec![],
        }
    }
}

impl Into<StatefulError<YamlSchemaError>> for YamlSchemaError {
    fn into(self) -> StatefulError<YamlSchemaError> {
        StatefulError {
            error: self,
            path: vec![],
        }
    }
}

impl<'a> Into<StatefulError<YamlValidationError<'a>>> for StringValidationError {
    fn into(self) -> StatefulError<YamlValidationError<'a>> {
        StatefulError {
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
    #[error(
        "input string contained multiple documents, but was attempted parsed as a single schema"
    )]
    MultipleSchemas,
    #[error("input string contained no yaml documents, or was empty")]
    NoSchema,
}

pub(crate) trait OptionalField<T> {
    fn into_optional(self) -> Result<Option<T>, StatefulError<YamlSchemaError>>;
}

impl<T> OptionalField<T> for Result<T, StatefulError<YamlSchemaError>> {
    fn into_optional(self) -> Result<Option<T>, StatefulError<YamlSchemaError>> {
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
    #[error("error reading yaml: {0}")]
    YamlScanError(#[from] ScanError),
    #[error("integer validation error: {0}")]
    IntegerValidationError(#[from] IntegerValidationError),
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
pub enum IntegerValidationError {
    #[error("integer too big, max is {0}, but value is {1}")]
    TooBig(i64, i64),
    #[error("integer too small, min is {0}, but value is {1}")]
    TooSmall(i64, i64),
}

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
