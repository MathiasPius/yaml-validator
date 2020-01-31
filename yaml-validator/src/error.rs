use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("error")]
    Bad,
}