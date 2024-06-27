use std::{borrow::Cow, fmt};

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
        #[cfg(feature = "hyper")]
        Hyper(hyper::Error),
        #[cfg(feature = "serde_json")]
        SerdeJson(serde_json::Error),
        #[cfg(feature = "serde_urlencoded")]
        SerdeUrlencoded(serde_urlencoded::ser::Error),
        #[cfg(feature = "client-hyper")]
        ClientHyper(hyper_util::client::legacy::Error),
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
