use std::borrow::Cow;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use crate::util::ok;
use crate::Error;
use http::header::CONTENT_TYPE;
pub use http::response::Builder;
pub use http::response::Response;
use http::HeaderValue;
use http_body_util::BodyExt;
#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;
use std::str::FromStr;
pub trait ResponseExt {
    #[cfg(feature = "json")]
    fn json<T: DeserializeOwned>(self) -> impl Future<Output = crate::Result<Response<T>>> + Send;
    fn text(self) -> impl Future<Output = crate::Result<Response<String>>> + Send;
}

pub type TextDecodeFn = fn(Vec<u8>) -> Result<String, Box<dyn std::error::Error + Send>>;

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

impl<B> ResponseExt for Response<B>
where
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: std::error::Error + Send + 'static,
{
    #[cfg(feature = "json")]
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
}
