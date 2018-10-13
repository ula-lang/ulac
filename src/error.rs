use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub struct Error {
    error: String
}

impl Error {
    pub fn new(error: String) -> Self {
        Self {
            error
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, r#"Error("{}")"#, self.error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        Display::fmt(&self.error, f)
    }
}


impl From<String> for Error {
    fn from(error: String) -> Self {
        Self::new(error)
    }
}

impl From<Vec<String>> for Error {
    fn from(errors: Vec<String>) -> Self {
        Self::new(errors.join("\r\n"))
    }
}