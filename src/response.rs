use std::borrow::Cow;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use crate::util::ok;
use crate::Error;
use bytes::Buf;
use bytes::Bytes;
use http::header::CONTENT_TYPE;
pub use http::response::Builder;
pub use http::response::Response;
use http::HeaderValue;
use http_body_util::BodyDataStream;
use http_body_util::BodyExt;
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
use std::str::FromStr;

/// Extension trait for [`http::Response`].
pub trait ResponseExt<B>: Sized {
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    fn json<T: DeserializeOwned>(self) -> impl Future<Output = crate::Result<Response<T>>> + Send;
    fn text(self) -> impl Future<Output = crate::Result<Response<String>>> + Send;
    fn bytes(self) -> impl Future<Output = crate::Result<Response<Bytes>>> + Send;
    fn data_stream(self) -> Response<BodyDataStream<B>>;
    fn buffer(self) -> impl Future<Output = crate::Result<Response<impl Buf>>> + Send;
    #[cfg(feature = "hyper")]
    #[cfg_attr(docsrs, doc(cfg(feature = "hyper")))]
    fn hyper_upgrade(self) -> impl Future<Output = crate::Result<hyper::upgrade::Upgraded>> + Send;
}

pub type TextDecodeFn = fn(Vec<u8>) -> Result<String, Box<dyn std::error::Error + Send>>;

/// A collection of text decoders.
#[derive(Debug, Default, Clone)]
pub struct Decoders {
    inner: Arc<HashMap<Cow<'static, str>, TextDecodeFn>>,
}

impl Decoders {
    pub fn new(map: HashMap<Cow<'static, str>, TextDecodeFn>) -> Self {
        Decoders {
            inner: Arc::new(map),
        }
    }
}

impl<B> ResponseExt<B> for Response<B>
where
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: std::error::Error + Send + 'static,
{
    /// Deserialize the response body as json.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    async fn json<T: DeserializeOwned>(self) -> crate::Result<Response<T>> {
        use bytes::Buf;
        let (parts, body) = self.into_parts();
        let body = body
            .collect()
            .await
            .map_err(Error::custom_with_context("collecting body stream"))?
            .aggregate();
        let body = serde_json::from_reader::<_, T>(body.reader())
            .map_err(Error::custom_with_context("deserialize json body"))?;
        Ok(Response::from_parts(parts, body))
    }

    /// Deserialize the response body as text.
    ///
    /// This function will try to decode the body with the charset specified in the `Content-Type` header.
    ///
    /// In most cases, the charset is `utf-8`. If the charset is not `utf-8`, you should enable the `charset` feature.
    async fn text(self) -> crate::Result<Response<String>> {
        use mime::Mime;
        let (parts, body) = self.into_parts();
        let body = body
            .collect()
            .await
            .map_err(Error::custom_with_context("collecting body stream"))?
            .to_bytes();
        let mut string_body: Option<String> = None;
        'decode: {
            if let Some(mime_type) = parts
                .headers
                .get(CONTENT_TYPE)
                .and_then(ok(HeaderValue::to_str))
                .and_then(ok(Mime::from_str))
            {
                let charset = mime_type.get_param(mime::CHARSET);
                let custom_charset = match charset {
                    Some(mime::UTF_8) | None => break 'decode,
                    Some(custom_charset) => custom_charset,
                };
                #[cfg(feature = "charset")]
                {
                    use encoding_rs::Encoding;
                    if let Some(encoding) = Encoding::for_label(custom_charset.as_str().as_bytes())
                    {
                        string_body.replace(encoding.decode(&body).0.to_string());
                        break 'decode;
                    }
                }
                let Some(decoders) = parts.extensions.get::<Decoders>() else {
                    break 'decode;
                };
                let Some(decoder_fn) = decoders.inner.get(custom_charset.as_str()) else {
                    break 'decode;
                };
                string_body = Some(
                    (decoder_fn)(body.to_vec()).map_err(Error::with_context("decode text body"))?,
                );
            }
        }

        let string_body = match string_body {
            Some(string_body) => string_body,
            None => {
                String::from_utf8(body.to_vec()).map_err(Error::with_context("decode text body"))?
            }
        };

        Ok(Response::from_parts(parts, string_body))
    }

    /// Wrap the response body as a data stream.
    #[inline]
    fn data_stream(self) -> Response<BodyDataStream<B>> {
        let (parts, body) = self.into_parts();
        let body = BodyDataStream::new(body);
        Response::from_parts(parts, body)
    }

    /// Collect the response body as bytes.
    async fn bytes(self) -> crate::Result<Response<Bytes>> {
        let (parts, body) = self.into_parts();
        let body = body
            .collect()
            .await
            .map_err(Error::custom_with_context("collecting body stream"))?
            .to_bytes();
        Ok(Response::from_parts(parts, body))
    }

    /// Collect the response body as buffer.
    ///
    /// This function is useful when you want to deserialize the body in various ways.
    async fn buffer(self) -> crate::Result<Response<impl Buf>> {
        let (parts, body) = self.into_parts();
        let body = body
            .collect()
            .await
            .map_err(Error::custom_with_context("collecting body stream"))?
            .aggregate();
        Ok(Response::from_parts(parts, body))
    }

    #[cfg(feature = "hyper")]
    #[cfg_attr(docsrs, doc(cfg(feature = "hyper")))]
    /// Upgrade the connection to a different protocol with hyper.
    ///
    /// This function yield a asynchronous io. You can use this to create a websocket connection by using some websocket lib.
    async fn hyper_upgrade(self) -> crate::Result<hyper::upgrade::Upgraded> {
        hyper::upgrade::on(self)
            .await
            .map_err(Error::with_context("upgrade connection"))
    }
}
