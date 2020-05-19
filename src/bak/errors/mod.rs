use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct RshError {
    pub(crate) message: String
}

impl Display for RshError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}", self.message).as_str())
    }
}

impl <T: std::error::Error> From<T> for RshError {
    fn from(inner: T) -> Self {
        RshError {message: format!("{}", inner)}
    }
}

pub type EvalResult<T> = std::result::Result<T, RshError>;
pub type CompilerResult<T> = std::result::Result<T, RshError>;
