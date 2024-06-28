use crate::error::BodyError;
use bytes::Bytes;
#[cfg(feature = "stream")]
use futures_core::Stream;
use http_body_util::{combinators::UnsyncBoxBody, Empty, Full};

pub type DynBody = UnsyncBoxBody<Bytes, BodyError>;
pub fn full<B>(body: B) -> Full<Bytes>
where
    B: Into<Bytes>,
{
    let body = body.into();
    Full::new(body)
}

pub fn empty() -> Empty<Bytes> {
    Empty::new()
}

#[cfg(feature = "stream")]
pub fn stream<S>(stream: S) -> http_body_util::StreamBody<S>
where
    S: Stream<Item = Result<http_body::Frame<Bytes>, BodyError>> + Send + 'static,
{
    http_body_util::StreamBody::new(stream)
}
