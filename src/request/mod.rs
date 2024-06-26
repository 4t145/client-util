use http::header::CONTENT_TYPE;
pub use http::request::Builder;
use http::HeaderValue;
pub use http::Request;
use hyper::body::Bytes;
use serde::Serialize;

pub trait TryIntoRequest {
    type Error: std::error::Error + Send + 'static;
    type Body: hyper::body::Body + Send;
    fn try_into_request(self) -> Result<Request<Self::Body>, Self::Error>;
}

pub trait RequestExt {
    fn json<B: Serialize + ?Sized>(self, body: &B) -> crate::Result<Request<Bytes>>;
    fn plain_text(self, body: impl Into<Bytes>) -> Request<Bytes>;
    fn query<Q: Serialize + ?Sized>(self, query: &Q) -> crate::Result<Request<Bytes>>;
}

impl<T> RequestExt for Request<T> {
    fn json<B: Serialize + ?Sized>(self, body: &B) -> crate::Result<Request<Bytes>> {
        let json_body =
            serde_json::to_vec(&body).map_err(crate::Error::with_context("serialize json body"))?;
        let (mut parts, _) = self.into_parts();
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );
        let body = Bytes::from(json_body);
        let request = Request::from_parts(parts, body);
        Ok(request)
    }
    fn plain_text(self, body: impl Into<Bytes>) -> Request<Bytes> {
        let (mut parts, _) = self.into_parts();
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
        );
        Request::from_parts(parts, body.into())
    }
    fn query<Q: Serialize + ?Sized>(self, query: &Q) -> crate::Result<Request<Bytes>> {
        let query = serde_urlencoded::to_string(&query)
            .map_err(crate::Error::with_context("serialize query string"))?;
        let (mut parts, body) = self.into_parts();
        let parts = parts.uri.into_parts();
        todo!()
    }
}
