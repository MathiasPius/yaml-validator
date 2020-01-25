use thiserror::Error;

pub type Result<'a> = std::result::Result<(), StatefulResult<'a>>;

pub struct StatefulResult<'a> {
    pub error: YamlValidationError<'a>,
    pub path: Vec<String>,
}

impl<'a> std::fmt::Display for StatefulResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in self.path.iter().rev() {
            write!(f, "{}", segment)?;
        }
        write!(f, ": {}", self.error)
    }
}

pub trait PathContext<'a> {
    fn prepend(self, segment: String) -> Self;
}

impl<'a> PathContext<'a> for Result<'a> {
    fn prepend(self, segment: String) -> Self {
        self.map_err(|mut state| {
            state.path.push(segment);
            state
        })
    }
}

impl<'a> Into<StatefulResult<'a>> for YamlValidationError<'a> {
    fn into(self) -> StatefulResult<'a> {
        StatefulResult {
            error: self,
            path: vec![],
        }
    }
}

impl<'a> Into<StatefulResult<'a>> for StringValidationError {
    fn into(self) -> StatefulResult<'a> {
        StatefulResult {
            error: self.into(),
            path: vec![],
        }
    }
}

impl<'a> Into<StatefulResult<'a>> for DictionaryValidationError<'a> {
    fn into(self) -> StatefulResult<'a> {
        StatefulResult {
            error: self.into(),
            path: vec![],
        }
    }
}

#[derive(Error, Debug)]
pub enum YamlValidationError<'a> {
    #[error("number validation error: {0}")]
    NumberValidationError(#[from] NumberValidationError),
    #[error("string validation error: {0}")]
    StringValidationError(#[from] StringValidationError),
    #[error("list validation error: {0}")]
    ListValidationError(#[from] ListValidationError),
    #[error("dictionary validation error: {0}")]
    DictionaryValidationError(DictionaryValidationError<'a>),
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
pub enum DictionaryValidationError<'a> {
    #[error("key type error: `{0}`")]
    KeyValidationError(Box<YamlValidationError<'a>>),
    #[error("key type error: `{0}`")]
    ValueValidationError(Box<YamlValidationError<'a>>),
}

impl<'a> From<DictionaryValidationError<'a>> for YamlValidationError<'a> {
    fn from(e: DictionaryValidationError<'a>) -> Self {
        YamlValidationError::DictionaryValidationError(e)
    }
}

#[derive(Error, Debug)]
pub enum ObjectValidationError {}
