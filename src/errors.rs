use std::fmt::Formatter;
use std::error::Error;

pub struct InterpreterError {
    pub(crate) message: String
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl std::fmt::Debug for InterpreterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl <T: Error> From<T> for InterpreterError {
    fn from(error: T) -> Self {
        InterpreterError{message:format!("{}", error)}
    }
}

pub type InterpreterResult<T> = std::result::Result<T, InterpreterError>;