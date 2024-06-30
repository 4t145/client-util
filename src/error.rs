use std::{borrow::Cow, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    context: Cow<'static, str>,
}

impl From<http::Error> for Error {
    fn from(val: http::Error) -> Self {
        Error::new(ErrorKind::Http(val), "unknown")
    }
}

#[derive(Debug)]
pub struct BodyError(pub(crate) Box<dyn std::error::Error + Send + Sync + 'static>);

impl std::fmt::Display for BodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BodyError {}
#[derive(Debug)]
pub struct ClientError(Box<dyn std::error::Error + Send + Sync + 'static>);

impl<E> From<E> for ClientError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(val: E) -> Self {
        ClientError(Box::new(val))
    }
}

impl From<ClientError> for Box<dyn std::error::Error + Send + Sync> {
    fn from(val: ClientError) -> Self {
        val.0
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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
    /// Create a new error with a context.
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
    /// Create a new custom error with a context.
    pub fn custom_with_context<C, E>(context: C) -> impl FnOnce(E) -> Self
    where
        C: Into<Cow<'static, str>>,
        E: std::error::Error + Send + 'static,
    {
        move |error| Error {
            kind: ErrorKind::unknown(error),
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

macro_rules! error_kinds {
    ($Ident: ident {
        $(
            $(#[$meta:meta])*
            $Variant: ident($Inner: ty)
        ),* $(,)?
    }) => {
        #[derive(Debug)]
        pub enum $Ident {
            $(
                $(#[$meta])*
                $Variant($Inner),
            )*
        }
        $(
            $(#[$meta])*
            impl From<$Inner> for $Ident {
                fn from(val: $Inner) -> Self {
                    $Ident::$Variant(val)
                }
            }
        )*
        impl fmt::Display for ErrorKind {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        $(#[$meta])*
                        ErrorKind::$Variant(e) => {
                            write!(f, "{}: {}", stringify!($Variant), e)
                        }
                    )*
                }
            }
        }
    };
}

error_kinds! {
    ErrorKind {
        Http(http::Error),
        InvalidUri(http::uri::InvalidUri),
        InvalidUriParts(http::uri::InvalidUriParts),
        Utf8DecodeError(std::string::FromUtf8Error),
        Unknown(Box<dyn std::error::Error + Send>),
        MimeParse(mime::FromStrError),
        InvalidHeaderValue(http::header::InvalidHeaderValue),
        Body(BodyError),
        Client(ClientError),
        #[cfg(feature = "hyper")]
        Hyper(hyper::Error),
        #[cfg(feature = "serde_json")]
        SerdeJson(serde_json::Error),
        #[cfg(feature = "serde_urlencoded")]
        SerdeUrlencoded(serde_urlencoded::ser::Error),
    }
}

impl ErrorKind {
    pub fn unknown<E>(error: E) -> Self
    where
        E: std::error::Error + Send + 'static,
    {
        ErrorKind::Unknown(Box::new(error))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] when [{}]", self.kind, self.context)
    }
}
