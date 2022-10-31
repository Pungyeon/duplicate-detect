use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error{
            msg: err.to_string(),
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error{
            msg: s,
        }
    }
}