use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("field `{0}` is in an invalid format")]
    IncorrectFormat(String),
    #[error("field `{0}` is missing")]
    MissingField(String),
    #[error("field `{0}`'s value of `{1}` is out of range")]
    OutOfRange(String, u8),
    #[error("unknown parsing error")]
    ParseFailure,
}
