use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct State<'a> {
    path: Vec<&'a str>,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        State { path: vec![] }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SchemaErrorKind<'schema> {
    #[error("wrong type, expected {expected} got {actual}")]
    WrongType {
        expected: &'static str,
        actual: &'schema str,
    },
    #[error("field {field} missing")]
    FieldMissing { field: &'schema str },
    #[error("field {field} is not specified in the schema")]
    ExtraField { field: &'schema str },
    #[error("unknown type specified: {unknown_type}")]
    UnknownType { unknown_type: &'schema str },
    #[error("multiple errors were encountered: {errors:?}")]
    Multiple { errors: Vec<SchemaError<'schema>> },
}

#[derive(Debug, PartialEq, Eq)]
pub struct SchemaError<'schema> {
    pub kind: SchemaErrorKind<'schema>,
    pub state: State<'schema>,
}

impl<'schema> SchemaError<'schema> {
    pub fn add_path(mut self, path: &'schema str) -> Self {
        self.state.path.push(path);
        self
    }
}

impl<'schema> SchemaErrorKind<'schema> {
    pub fn with_path(self, path: Vec<&'schema str>) -> SchemaError<'schema> {
        SchemaError {
            kind: self,
            state: State { path },
        }
    }
}

pub fn add_err_path<'schema>(
    path: &'schema str,
) -> impl Fn(SchemaError<'schema>) -> SchemaError<'schema> {
    move |err: SchemaError<'schema>| -> SchemaError<'schema> { err.add_path(path) }
}

pub fn optional<'schema, T>(
    default: T,
) -> impl FnOnce(SchemaError<'schema>) -> Result<T, SchemaError<'schema>> {
    move |err: SchemaError<'schema>| -> Result<T, SchemaError<'schema>> {
        match err.kind {
            SchemaErrorKind::FieldMissing { .. } => Ok(default),
            _ => Err(err),
        }
    }
}

impl<'schema> Into<SchemaError<'schema>> for SchemaErrorKind<'schema> {
    fn into(self) -> SchemaError<'schema> {
        SchemaError {
            kind: self,
            state: State::default(),
        }
    }
}
