use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum PathSegment<'a> {
    Name(&'a str),
    Index(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub struct State<'a> {
    path: Vec<PathSegment<'a>>,
}

impl<'a> std::fmt::Display for State<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in self.path.iter().rev() {
            match segment {
                PathSegment::Name(name) => write!(f, ".{}", name)?,
                PathSegment::Index(index) => write!(f, "[{}]", index)?,
            };
        }

        Ok(())
    }
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
    #[error("field '{field}' missing")]
    FieldMissing { field: &'schema str },
    #[error("field '{field}' is not specified in the schema")]
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
    fn flatten(&self, fmt: &mut std::fmt::Formatter<'_>, root: String) -> std::fmt::Result {
        match &self.kind {
            SchemaErrorKind::Multiple { errors } => {
                for err in errors {
                    err.flatten(fmt, format!("{}{}", root, self.state))?;
                }
            }
            err => writeln!(fmt, "{}{}: {}", root, self.state, err)?,
        }

        Ok(())
    }
}

impl<'a> std::fmt::Display for SchemaError<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.flatten(fmt, "#".to_string())
    }
}

#[cfg(test)]
impl<'schema> SchemaErrorKind<'schema> {
    pub fn with_path(self, path: Vec<PathSegment<'schema>>) -> SchemaError<'schema> {
        SchemaError {
            kind: self,
            state: State { path },
        }
    }
}

pub fn add_path_name<'schema>(
    path: &'schema str,
) -> impl Fn(SchemaError<'schema>) -> SchemaError<'schema> {
    move |mut err: SchemaError<'schema>| -> SchemaError<'schema> {
        err.state.path.push(PathSegment::Name(path));
        err
    }
}

pub fn add_path_index<'schema>(
    index: usize,
) -> impl Fn(SchemaError<'schema>) -> SchemaError<'schema> {
    move |mut err: SchemaError<'schema>| -> SchemaError<'schema> {
        err.state.path.push(PathSegment::Index(index));
        err
    }
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
