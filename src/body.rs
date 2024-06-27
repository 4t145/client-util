use bytes::Bytes;
use futures_core::Stream;
use http_body::Frame;
use http_body_util::{combinators::UnsyncBoxBody, BodyExt, Empty, Full, StreamBody};
use tower_http::BoxError;

pub type Body = UnsyncBoxBody<Bytes, BoxError>;
pub fn full<B>(body: B) -> Body
where
    B: Into<Bytes>,
{
    let body = body.into();
    Full::new(body).map_err(Into::into).boxed_unsync()
}

pub fn empty() -> Body {
    Empty::new().map_err(Into::into).boxed_unsync()
}

pub fn stream<S>(stream: S) -> Body
where
    S: Stream<Item = Result<Frame<Bytes>, BoxError>> + Send + 'static,
{
    StreamBody::new(stream).boxed_unsync()
}
