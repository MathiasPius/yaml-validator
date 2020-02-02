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
pub enum SchemaErrorKind<'a> {
    #[error("wrong type, expected {expected} got {actual}")]
    WrongType {
        expected: &'static str,
        actual: &'a str,
    },
    #[error("field '{field}' missing")]
    FieldMissing { field: &'a str },
    #[error("field '{field}' is not specified in the schema")]
    ExtraField { field: &'a str },
    #[error("unknown type specified: {unknown_type}")]
    UnknownType { unknown_type: &'a str },
    #[error("multiple errors were encountered: {errors:?}")]
    Multiple { errors: Vec<SchemaError<'a>> },
}

#[derive(Debug, PartialEq, Eq)]
pub struct SchemaError<'a> {
    pub kind: SchemaErrorKind<'a>,
    pub state: State<'a>,
}

impl<'a> SchemaError<'a> {
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
impl<'a> SchemaErrorKind<'a> {
    pub fn with_path(self, path: Vec<PathSegment<'a>>) -> SchemaError<'a> {
        SchemaError {
            kind: self,
            state: State { path },
        }
    }
}

pub fn add_path_name<'a>(
    path: &'a str,
) -> impl Fn(SchemaError<'a>) -> SchemaError<'a> {
    move |mut err: SchemaError<'a>| -> SchemaError<'a> {
        err.state.path.push(PathSegment::Name(path));
        err
    }
}

pub fn add_path_index<'a>(
    index: usize,
) -> impl Fn(SchemaError<'a>) -> SchemaError<'a> {
    move |mut err: SchemaError<'a>| -> SchemaError<'a> {
        err.state.path.push(PathSegment::Index(index));
        err
    }
}

pub fn optional<'a, T>(
    default: T,
) -> impl FnOnce(SchemaError<'a>) -> Result<T, SchemaError<'a>> {
    move |err: SchemaError<'a>| -> Result<T, SchemaError<'a>> {
        match err.kind {
            SchemaErrorKind::FieldMissing { .. } => Ok(default),
            _ => Err(err),
        }
    }
}

impl<'a> Into<SchemaError<'a>> for SchemaErrorKind<'a> {
    fn into(self) -> SchemaError<'a> {
        SchemaError {
            kind: self,
            state: State::default(),
        }
    }
}
