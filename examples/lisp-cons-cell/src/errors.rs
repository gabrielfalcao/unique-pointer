use crate::{impl_error, Traceback};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ErrorType {
    RuntimeError,
}
impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::RuntimeError => "RuntimeError",
            }
        )
    }
}
impl_error!(Error, ErrorType);
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::new(e, ErrorType::IOError)
    }
}
impl From<iocore::Error> for Error {
    fn from(e: iocore::Error) -> Self {
        Error::new(e, ErrorType::IOError)
    }
}

impl From<minilisp_parser::Error<'_>> for Error {
    fn from(e: minilisp_parser::Error<'_>) -> Self {
        Error::new(e, ErrorType::ParserError)
    }
}
