use crate::request::BuildRequestError;

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Http(#[from] http::Error),
    // InvalidUri(#[from] http::uri::InvalidUri),
    // InvalidUriParts(#[from] http::uri::InvalidUriParts),
    // Utf8DecodeError(#[from] std::string::FromUtf8Error),
    // Unknown(#[from] Box<dyn std::error::Error + Send>),
    // MimeParse(#[from] mime::FromStrError),
    // InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("body error: {0}")]
    Body(crate::body::StdError),
    #[error("request build error: {0}")]
    BuildRequest(#[from] BuildRequestError),
}

