use std::{borrow::Cow, fmt};

pub mod body;
pub mod header;
pub mod request;
pub mod response;
mod util;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    context: Cow<'static, str>,
}

impl Error {
    pub fn new<E, C>(kind: E, context: C) -> Self
    where
        E: Into<ErrorKind>,
        C: Into<Cow<'static, str>>,
    {
        Error {
            kind: kind.into(),
            context: context.into(),
        }
    }
    pub fn with_context<C, K>(context: C) -> impl FnOnce(K) -> Self
    where
        C: Into<Cow<'static, str>>,
        K: Into<ErrorKind>,
    {
        move |kind| Error {
            kind: kind.into(),
            context: context.into(),
        }
    }
    pub fn custom_with_context<C, E>(context: C) -> impl FnOnce(E) -> Self
    where
        C: Into<Cow<'static, str>>,
        E: std::error::Error + Send + 'static,
    {
        move |error| Error {
            kind: ErrorKind::custom(error),
            context: context.into(),
        }
    }
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn context(&self) -> &str {
        &self.context
    }

    pub fn into_inner(self) -> ErrorKind {
        self.kind
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Hyper(hyper::Error),
    Http(http::Error),
    SerdeJson(serde_json::Error),
    SerdeUrlencoded(serde_urlencoded::ser::Error),
    Custom(Box<dyn std::error::Error + Send>),
    Utf8DecodeError(std::string::FromUtf8Error),
}

impl ErrorKind {
    pub fn custom<E>(error: E) -> Self
    where
        E: std::error::Error + Send + 'static,
    {
        ErrorKind::Custom(Box::new(error))
    }
}

impl From<hyper::Error> for ErrorKind {
    fn from(val: hyper::Error) -> Self {
        ErrorKind::Hyper(val)
    }
}

impl From<http::Error> for ErrorKind {
    fn from(val: http::Error) -> Self {
        ErrorKind::Http(val)
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(val: serde_json::Error) -> Self {
        ErrorKind::SerdeJson(val)
    }
}

impl From<std::string::FromUtf8Error> for ErrorKind {
    fn from(val: std::string::FromUtf8Error) -> Self {
        ErrorKind::Utf8DecodeError(val)
    }
}

impl From<serde_urlencoded::ser::Error> for ErrorKind {
    fn from(val: serde_urlencoded::ser::Error) -> Self {
        ErrorKind::SerdeUrlencoded(val)
    }
}

impl From<Box<dyn std::error::Error + std::marker::Send>> for ErrorKind {
    fn from(val: Box<dyn std::error::Error + std::marker::Send>) -> Self {
        ErrorKind::Custom(val)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Hyper(e) => {
                write!(f, "hyper::Error: {}", e)
            }
            ErrorKind::Http(e) => {
                write!(f, "http::Error: {}", e)
            }
            ErrorKind::SerdeJson(e) => {
                write!(f, "serde_json::Error: {}", e)
            }
            ErrorKind::Utf8DecodeError(e) => {
                write!(f, "Utf8DecodeError: {}", e)
            }
            ErrorKind::SerdeUrlencoded(e) => {
                write!(f, "serde_urlencoded::ser::Error: {}", e)
            }
            ErrorKind::Custom(e) => {
                write!(f, "Custom Error: {}", e)
            }
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] when [{}]", self.kind, self.context)
    }
}
