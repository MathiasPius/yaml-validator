use thiserror::Error;

pub type Result<'a> = std::result::Result<(), YamlValidationError<'a>>;

#[derive(Error, Debug)]
pub enum YamlValidationError<'a> {
    #[error("number validation error: {0}")]
    NumberValidationError(#[from] NumberValidationError),
    #[error("string validation error: {0}")]
    StringValidationError(#[from] StringValidationError),
    #[error("list validation error: {0}")]
    ListValidationError(#[from] ListValidationError),
    #[error("dictionary validation error: {0}")]
    DictionaryValidationError(#[from] DictionaryValidationError),
    #[error("object validation error: {0}")]
    ObjectValidationError(#[from] ObjectValidationError),
    #[error("wrong type, expected `{0}` got `{1:?}`")]
    WrongType(&'static str, &'a serde_yaml::Value),
    #[error("missing field, `{0}` not found")]
    MissingField(&'a str),
}

#[derive(Error, Debug)]
pub enum NumberValidationError {}

#[derive(Error, Debug)]
pub enum StringValidationError {
    #[error("string too long, max is {0}, but string is {1}")]
    TooLong(usize, usize),
    #[error("string too short, min is {0}, but string is {1}")]
    TooShort(usize, usize),
}

#[derive(Error, Debug)]
pub enum ListValidationError {}

#[derive(Error, Debug)]
pub enum DictionaryValidationError {}

#[derive(Error, Debug)]
pub enum ObjectValidationError {}
