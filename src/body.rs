use bytes::Bytes;
#[cfg(feature = "stream")]
use futures_core::Stream;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
pub type Body = BoxBody<Bytes, crate::error::BoxError>;

/// Create a new full body.
pub fn full<B>(body: B) -> Full<Bytes>
where
    B: Into<Bytes>,
{
    let body = body.into();
    Full::new(body)
}

pub fn boxed_full<B>(body: B) -> Body
where
    B: Into<Bytes>,
{
    full(body).map_err(crate::util::never).boxed()
}

/// Create a new empty body.
pub fn empty() -> Empty<Bytes> {
    Empty::new()
}

pub fn boxed_empty() -> Body {
    empty().map_err(crate::util::never).boxed()
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
pub fn stream<S>(stream: S) -> http_body_util::StreamBody<S>
where
    S: Stream<Item = Result<http_body::Frame<Bytes>, crate::error::BoxError>> + Send + 'static,
{
    http_body_util::StreamBody::new(stream)
}

pub fn boxed_stream<S>(s: S) -> Body
where
    S: Stream<Item = Result<http_body::Frame<Bytes>, crate::error::BoxError>>
        + Send
        + Sync
        + 'static,
{
    stream(s).map_err(crate::util::never).boxed()
}

#[cfg(feature = "io-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "io-tokio")))]
pub fn tokio_async_read<R>(reader: R) -> Body
where
    R: tokio::io::AsyncRead + Send + Sync + 'static,
{
    use futures_util::TryStreamExt;
    use http_body::Frame;
    use http_body_util::BodyExt;

    let stream = tokio_util::io::ReaderStream::new(reader);
    let body = http_body_util::StreamBody::new(
        stream
            .map_ok(Frame::data)
            .map_err(|e| Box::new(e) as crate::error::BoxError),
    );
    body.boxed()
}
