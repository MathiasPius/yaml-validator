//use yaml_validator::{StatefulError, YamlSchemaError};
use yaml_validator::{ScanError, SchemaError};

#[derive(Debug)]
pub enum Error {
    FileError(String),
    ValidationError(String),
    YamlError(String),
    InputError(String),
    Multiple(Vec<Error>),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::FileError(format!("{}", e))
    }
}

impl From<ScanError> for Error {
    fn from(e: ScanError) -> Self {
        Error::YamlError(format!("{}", e))
    }
}

impl<'a> From<SchemaError<'a>> for Error {
    fn from(e: SchemaError<'a>) -> Self {
        Error::ValidationError(format!("{}", e))
    }
}

/*
impl From<YamlSchemaError> for Error {
    fn from(e: YamlSchemaError) -> Self {
        Error::YamlError(format!("{}", e))
    }
}

impl<E> From<StatefulError<E>> for Error
where
    E: std::fmt::Display,
{
    fn from(e: StatefulError<E>) -> Self {
        Error::ValidationError(format!("{}: {}", e.path.join(""), e.error))
    }
}
*/

impl<'a> std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FileError(e) => writeln!(f, "{}", e),
            Error::ValidationError(e) => writeln!(f, "{}", e),
            Error::YamlError(e) => writeln!(f, "{}", e),
            Error::InputError(e) => writeln!(f, "{}", e),
            Error::Multiple(e) => {
                for err in e {
                    write!(f, "{}", err)?;
                }
                Ok(())
            }
        }
    }
}
