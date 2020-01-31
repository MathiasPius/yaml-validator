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
    #[error("schema {expected} descriptor is a {actual} not a hash")]
    DescriptorNotHash {
        expected: &'static str,
        actual: &'schema str,
    },
    #[error("wrong type, expected {expected} got {actual}")]
    WrongType {
        expected: &'static str,
        actual: &'schema str,
    },
    #[error("field {field} missing")]
    FieldMissing { field: &'static str },
    #[error("unknown type specified: {unknown_type}")]
    UnknownType { unknown_type: &'schema str },
    //#[error("multiple errors were encountered: {errors:?}")]
    //Multiple { errors: Vec<SchemaError<'schema>> },
}

#[derive(Debug, PartialEq, Eq)]
pub struct SchemaError<'schema> {
    pub kind: SchemaErrorKind<'schema>,
    pub state: State<'schema>,
}

impl<'schema> Into<SchemaError<'schema>> for SchemaErrorKind<'schema> {
    fn into(self) -> SchemaError<'schema> {
        SchemaError {
            kind: self,
            state: State::default(),
        }
    }
}
