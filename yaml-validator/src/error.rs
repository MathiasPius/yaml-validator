#![macro_use]

use thiserror::Error;

#[cfg(feature = "smallvec-optimization")]
pub type PathVec<'a> = smallvec::SmallVec<[PathSegment<'a>;8]>;
#[cfg(not(feature = "smallvec-optimization"))]
pub type PathVec<'a> = Vec<PathSegment<'a>>;

#[cfg(test)]
#[cfg(feature = "smallvec-optimization")]
macro_rules! path{
    ( $( $x:expr ),* ) => {
        smallvec![
            $(PathSegment::from($x),)*
        ]
    }
}

#[cfg(test)]
#[cfg(not(feature = "smallvec-optimization"))]
macro_rules! path{
    ( $( $x:expr ),* ) => {
        vec![
            $(PathSegment::from($x),)*
        ]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PathSegment<'a> {
    Name(&'a str),
    Index(usize),
}

impl<'a> From<&'a str> for PathSegment<'a> {
    fn from(name: &'a str) -> Self {
        PathSegment::Name(name)
    }
}

impl<'a> From<usize> for PathSegment<'a> {
    fn from(index: usize) -> Self {
        PathSegment::Index(index)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct State<'a> {
    path: PathVec<'a>,
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
        State { path: PathVec::new() }
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
    #[error("schema '{uri}' references was not found")]
    UnknownSchema { uri: &'a str },
}

/// A wrapper type around SchemaErrorKind containing path information about where the error occurred.
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
    pub fn with_path(self, path: PathVec<'a>) -> SchemaError<'a> {
        SchemaError {
            kind: self,
            state: State { path },
        }
    }
}

pub fn add_path_name<'a>(path: &'a str) -> impl Fn(SchemaError<'a>) -> SchemaError<'a> {
    move |mut err: SchemaError<'a>| -> SchemaError<'a> {
        err.state.path.push(PathSegment::Name(path));
        err
    }
}

pub fn add_path_index<'a>(index: usize) -> impl Fn(SchemaError<'a>) -> SchemaError<'a> {
    move |mut err: SchemaError<'a>| -> SchemaError<'a> {
        err.state.path.push(PathSegment::Index(index));
        err
    }
}

pub fn optional<'a, T>(default: T) -> impl FnOnce(SchemaError<'a>) -> Result<T, SchemaError<'a>> {
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

#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::utils::load_simple;
    use crate::{Context, Validate};
    use std::convert::TryFrom;
    #[test]
    fn test_error_path() {
        let yaml = load_simple(
            r#"
            items:
              test:
                type: integer
              something:
                type: object
                items:
                  level2:
                    type: object
                    items:
                      leaf: hello
            "#,
        );

        let err = SchemaObject::try_from(&yaml).unwrap_err();

        assert_eq!(
            format!("{}", err),
            "#.items.something.items.level2.items.leaf: field \'type\' missing\n",
        );
    }

    #[test]
    fn test_error_path_validation() {
        let yaml = load_simple(
            r#"
            items:
              test:
                type: integer
              something:
                type: object
                items:
                  level2:
                    type: array
                    items:
                      type: object
                      items:
                        num:
                          type: integer
            "#,
        );

        let schema = SchemaObject::try_from(&yaml).unwrap();
        let document = load_simple(
            r#"
            test: 20
            something:
              level2:
                - num: abc
                - num:
                    hash: value
                - num:
                    - array: hello
                - num: 10
                - num: jkl
            "#,
        );
        let ctx = Context::default();
        let err = schema.validate(&ctx, &document).unwrap_err();

        assert_eq!(
            format!("{}", err),
            r#"#.something.level2[0].num: wrong type, expected integer got string
#.something.level2[1].num: wrong type, expected integer got hash
#.something.level2[2].num: wrong type, expected integer got array
#.something.level2[4].num: wrong type, expected integer got string
"#
        );
    }
}
