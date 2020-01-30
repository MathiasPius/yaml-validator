use yaml_validator::{StatefulError, YamlSchemaError};

pub enum Error {
    FileError(String),
    ValidationError(String),
    YamlError(String),
    InputError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::FileError(format!("{}", e))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::YamlError(format!("{}", e))
    }
}

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

impl<'a> std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FileError(e) => write!(f, "{}", e),
            Error::ValidationError(e) => write!(f, "{}", e),
            Error::YamlError(e) => write!(f, "{}", e),
            Error::InputError(e) => write!(f, "{}", e),
        }
    }
}
