use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct CustomError {
    pub message: String
}

impl CustomError {
    pub fn new(message: &str) -> Self {
        Self {message: message.to_string()}
    }
}

impl Display for CustomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Custom error : {}", self.message)
    }
}

impl std::error::Error for CustomError {}
