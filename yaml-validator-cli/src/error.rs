use yaml_validator::{yaml_rust::ScanError, SchemaError};

#[derive(Debug)]
pub enum Error {
    FileError(String),
    ValidationError(String),
    YamlError(String),
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

impl<'a> std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FileError(e) => write!(f, "{}", e),
            Error::ValidationError(e) => write!(f, "{}", e),
            Error::YamlError(e) => write!(f, "{}", e),
            Error::Multiple(e) => {
                for err in e {
                    write!(f, "{}", err)?;
                }
                Ok(())
            }
        }
    }
}
