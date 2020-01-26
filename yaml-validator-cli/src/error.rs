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
