use yaml_validator::{yaml_rust::ScanError, SchemaError};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    File(String),
    Validation(String),
    Yaml(String),
    Multiple(Vec<Error>),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::File(format!("{}", e))
    }
}

impl From<ScanError> for Error {
    fn from(e: ScanError) -> Self {
        Error::Yaml(format!("{}", e))
    }
}

impl<'a> From<SchemaError<'a>> for Error {
    fn from(e: SchemaError<'a>) -> Self {
        Error::Validation(format!("{}", e))
    }
}

impl<'a> std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::File(e) => write!(f, "{}", e),
            Error::Validation(e) => write!(f, "{}", e),
            Error::Yaml(e) => write!(f, "{}", e),
            Error::Multiple(e) => {
                for err in e {
                    write!(f, "{}", err)?;
                }
                Ok(())
            }
        }
    }
}
