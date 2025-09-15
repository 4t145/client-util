use crate::request::BuildRequestError;

pub type Result<T> = std::result::Result<T, Error>;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("body error: {0}")]
    Body(#[source] BoxError),
    #[error("request build error: {0}")]
    BuildRequest(#[from] BuildRequestError),
    #[error("request send error: {0}")]
    SendRequest(#[source] BoxError),
    #[error("response error: {0}")]
    Response(#[from] crate::response::ResponseError),
    #[error("raw http error: {0}")]
    Http(#[from] http::Error),
}
