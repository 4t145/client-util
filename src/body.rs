use bytes::Bytes;
#[cfg(feature = "stream")]
use futures_core::Stream;
use http_body_util::{combinators::UnsyncBoxBody, Empty, Full};
pub type StdError = Box<dyn std::error::Error + Send + 'static>;
pub type Body = UnsyncBoxBody<Bytes, StdError>;

/// Create a new full body.
pub fn full<B>(body: B) -> Full<Bytes>
where
    B: Into<Bytes>,
{
    let body = body.into();
    Full::new(body)
}

/// Create a new empty body.
pub fn empty() -> Empty<Bytes> {
    Empty::new()
}

#[cfg(feature = "stream")]
pub fn stream<S>(stream: S) -> http_body_util::StreamBody<S>
where
    S: Stream<Item = Result<http_body::Frame<Bytes>, Body>> + Send + 'static,
{
    http_body_util::StreamBody::new(stream)
}

#[cfg(feature = "io-tokio")]
pub fn tokio_async_read<R>(reader: R) -> Body
where
    R: tokio::io::AsyncRead + Send + 'static,
{
    use futures_util::TryStreamExt;
    use http_body::Frame;
    use http_body_util::BodyExt;

    let stream = tokio_util::io::ReaderStream::new(reader);
    let body = http_body_util::StreamBody::new(
        stream.map_ok(Frame::data).map_err(|e| Box::new(e) as StdError),
    );
    body.boxed_unsync()
}
