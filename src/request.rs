#[cfg(feature = "multipart")]
#[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
mod multipart;
use bytes::Bytes;
use futures_util::TryFutureExt;
use http::uri::PathAndQuery;
use http::HeaderValue;
use http::Request;
use http::Response;
use http::{header::CONTENT_TYPE, Uri};
use http_body_util::combinators::BoxBody;
use http_body_util::{Empty, Full};
#[cfg(feature = "multipart")]
pub use multipart::*;
#[cfg(feature = "serde")]
use serde::Serialize;
use std::convert::Infallible;
use std::future::Future;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;

use crate::body::{empty, full};
use crate::client::ClientBody;

#[derive(Debug, thiserror::Error)]
pub enum BuildRequestError {
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    #[error("failed to serialize json body: {0}")]
    BuildJsonBody(#[from] serde_json::Error),
    #[cfg(feature = "form")]
    #[cfg_attr(docsrs, doc(cfg(feature = "form")))]
    #[error("failed to build form body: {0}")]
    BuildForm(#[from] BuildFormError),
    #[cfg(feature = "multipart")]
    #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
    #[error("failed to build multipart body: {0}")]
    BuildMultipart(#[from] BuildMultipartError),
    #[error("failed to build request path: {0}")]
    BuildPath(#[from] BuildPathError),
    #[cfg(feature = "query")]
    #[cfg_attr(docsrs, doc(cfg(feature = "query")))]
    #[error("failed to build request query: {0}")]
    BuildQuery(#[from] BuildQueryError),
    #[error("invalid uri: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("failed to build request: {0}")]
    HttpError(#[from] http::Error),
}

impl From<Infallible> for BuildRequestError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BuildPathError {
    #[error("invalid uri: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("invalid uri parts: {0}")]
    InvalidUriParts(#[from] http::uri::InvalidUriParts),
}

#[cfg(feature = "query")]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
#[derive(Debug, thiserror::Error)]
pub enum BuildQueryError {
    #[error("invalid uri: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("invalid uri parts: {0}")]
    InvalidUriParts(#[from] http::uri::InvalidUriParts),
    #[error("failed to serialize query string: {0}")]
    SerializeQuery(#[from] serde_urlencoded::ser::Error),
}

#[cfg(feature = "query")]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
#[derive(Debug, thiserror::Error)]
pub enum BuildMultipartError {
    #[error("invalid boundary header: {0}")]
    InvalidBoundaryHeader(#[from] http::header::InvalidHeaderValue),
    #[error("invalid mime type: {0}")]
    InvalidMime(#[from] mime::FromStrError),
}

#[cfg(feature = "form")]
#[cfg_attr(docsrs, doc(cfg(feature = "form")))]
#[derive(Debug, thiserror::Error)]
pub enum BuildFormError {
    #[error("failed to serialize form body: {0}")]
    SerializeForm(#[from] serde_urlencoded::ser::Error),
    #[error("invalid content type header: {0}")]
    InvalidContentTypeHeader(#[from] http::header::InvalidHeaderValue),
}

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
#[derive(Debug, thiserror::Error)]
pub enum BuildJsonBodyError {
    #[error("failed to serialize json body: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
pub struct RequestBuilder {
    parts: http::request::Parts,
}

impl Deref for RequestBuilder {
    type Target = http::request::Parts;
    fn deref(&self) -> &Self::Target {
        &self.parts
    }
}

impl DerefMut for RequestBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parts
    }
}

macro_rules! http_methods {
    ($fn: ident  $method: expr) => {
        pub fn $fn<T>(uri: T) -> Result<Self, BuildRequestError>
        where
            T: TryInto<Uri>,
            <T as TryInto<Uri>>::Error: Into<BuildRequestError>,
        {
            let mut this = Self::new();
            this.parts.method = $method;
            this.parts.uri = uri.try_into().map_err(Into::into)?;
            Ok(this)
        }
    };
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestBuilder {
    pub fn new() -> Self {
        let (parts, _) = http::Request::new(()).into_parts();
        Self { parts }
    }
    pub fn uri(mut self, uri: Uri) -> Self {
        self.parts.uri = uri;
        self
    }
    http_methods!(get http::Method::GET);
    http_methods!(post http::Method::POST);
    http_methods!(put http::Method::PUT);
    http_methods!(delete http::Method::DELETE);
    http_methods!(head http::Method::HEAD);
    http_methods!(patch http::Method::PATCH);
    http_methods!(options http::Method::OPTIONS);
    http_methods!(trace http::Method::TRACE);
    http_methods!(connect http::Method::CONNECT);

    pub fn method(mut self, method: http::Method) -> Self {
        self.parts.method = method;
        self
    }
    pub fn version(mut self, version: http::Version) -> Self {
        self.parts.version = version;
        self
    }
    pub fn body<B>(self, body: B) -> Result<Request<B>, BuildRequestError>
    where
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<crate::error::BoxError>,
    {
        let request = Request::from_parts(self.parts, body);
        Ok(request)
    }
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn json<T: Serialize + ?Sized>(
        self,
        body: &T,
    ) -> Result<Request<Full<Bytes>>, BuildRequestError> {
        let json_body = serde_json::to_vec(&body)?;
        let mut parts = self.parts;
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );
        let request = Request::from_parts(parts, Full::new(Bytes::from(json_body)));
        Ok(request)
    }
    #[cfg(feature = "multipart")]
    #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
    pub fn multipart(
        self,
        mut form: multipart::Form,
    ) -> Result<Request<crate::Body>, BuildRequestError> {
        let mut parts = self.parts;
        let boundary = form.boundary();
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&format!(
                "{}; boundary={}",
                mime::MULTIPART_FORM_DATA,
                boundary
            ))
            .map_err(BuildMultipartError::from)?,
        );
        if let Some(length) = form.compute_length() {
            parts.headers.insert(
                http::header::CONTENT_LENGTH,
                HeaderValue::from_str(&length.to_string())
                    .expect("content length is always valid HeaderValue"),
            );
        }
        let body = form.stream();
        Ok(Request::from_parts(parts, body))
    }
    /// Set the request body as form data.
    #[cfg(feature = "form")]
    #[cfg_attr(docsrs, doc(cfg(feature = "form")))]
    pub fn form<T: Serialize + ?Sized>(
        mut self,
        form: &T,
    ) -> Result<Request<Full<Bytes>>, BuildRequestError> {
        let body = serde_urlencoded::to_string(form).map_err(BuildFormError::from)?;
        self.parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_WWW_FORM_URLENCODED.as_ref()),
        );
        Ok(Request::from_parts(self.parts, full(body)))
    }
    pub fn plain_text(self, body: impl Into<Bytes>) -> Request<Full<Bytes>> {
        Request::from_parts(self.parts, full(body))
    }
    pub fn empty(self) -> Request<Empty<Bytes>> {
        Request::from_parts(self.parts, empty())
    }
    #[cfg(feature = "query")]
    #[cfg_attr(docsrs, doc(cfg(feature = "query")))]
    pub fn query<Q: Serialize + ?Sized>(mut self, query: &Q) -> Result<Self, BuildRequestError> {
        self.parts.uri = build_query_uri(self.parts.uri, query)?;
        Ok(self)
    }
    pub fn path(mut self, path: impl AsRef<str>) -> Result<Self, BuildRequestError> {
        let path = path.as_ref();
        self.parts.uri = build_path_uri(self.parts.uri, path)?;
        Ok(self)
    }
    pub fn headers(mut self, header_map: http::header::HeaderMap) -> Self {
        self.parts.headers.extend(header_map);
        self
    }
    pub fn header<V>(
        mut self,
        key: impl http::header::IntoHeaderName,
        value: V,
    ) -> Result<Self, BuildRequestError>
    where
        V: TryInto<HeaderValue>,
        V::Error: Into<BuildRequestError>,
    {
        self.parts
            .headers
            .insert(key, value.try_into().map_err(Into::into)?);
        Ok(self)
    }
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: std::fmt::Display,
        P: std::fmt::Display,
        Self: Sized,
    {
        let header_value = crate::util::basic_auth(username, password);
        self.header(http::header::AUTHORIZATION, header_value)
            .expect("base64 should always be a valid header value")
    }
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: std::fmt::Display,
    {
        let header_value = crate::util::bearer_auth(token);
        self.header(http::header::AUTHORIZATION, header_value)
            .expect("base64 should always be a valid header value")
    }
}

/// Extension trait for [`http::Request`].
pub trait RequestExt<B>: Sized {
    fn with_version(self, version: http::Version) -> Request<B>;
    fn with_method(self, method: http::Method) -> Request<B>;
    fn with_header<K>(self, key: K, value: http::header::HeaderValue) -> Request<B>
    where
        K: http::header::IntoHeaderName;
    fn with_headers(self, header_map: http::header::HeaderMap) -> Request<B>;
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    fn with_basic_auth<U, P>(self, username: U, password: Option<P>) -> Request<B>
    where
        U: std::fmt::Display,
        P: std::fmt::Display,
        Self: Sized,
    {
        let header_value = crate::util::basic_auth(username, password);
        self.with_header(http::header::AUTHORIZATION, header_value)
    }

    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    fn with_bearer_auth<T>(self, token: T) -> Request<B>
    where
        T: std::fmt::Display,
    {
        let header_value = crate::util::bearer_auth(token);
        self.with_header(http::header::AUTHORIZATION, header_value)
    }

    fn send<S, R>(self, client: S) -> impl Future<Output = crate::Result<S::Response>> + Send
    where
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<crate::error::BoxError>,
        S: tower_service::Service<Request<ClientBody>, Response = Response<R>> + Send + Sync,
        R: http_body::Body + Send + Sync + 'static,
        <S as tower_service::Service<Request<ClientBody>>>::Error: Into<crate::error::BoxError>,
        <S as tower_service::Service<Request<ClientBody>>>::Future: Send;
}

impl<B> RequestExt<B> for Request<B>
where
    B: Send,
{
    /// Set the request HTTP version.
    #[inline]
    fn with_version(mut self, version: http::Version) -> Request<B> {
        *self.version_mut() = version;
        self
    }

    /// Set the request method.
    #[inline]
    fn with_method(mut self, method: http::Method) -> Request<B> {
        *self.method_mut() = method;
        self
    }

    /*
    // I think we may don't need to modify schema?
    fn schema(self, schema: Scheme) -> crate::Result<Request<B>> {
        let (mut parts, body) = self.into_parts();
        let mut uri_parts = parts.uri.into_parts();
        uri_parts.scheme = Some(schema);
        parts.uri = Uri::from_parts(uri_parts).map_err(crate::Error::with_context(
            "reconstruct uri with new schema",
        ))?;
        Ok(Request::from_parts(parts, body))
    }
    */
    /// Set a request header.
    #[inline]
    fn with_header<K>(mut self, key: K, value: http::header::HeaderValue) -> Request<B>
    where
        K: http::header::IntoHeaderName,
    {
        self.headers_mut().insert(key, value);
        self
    }

    /// Extend multiple request headers.
    #[inline]
    fn with_headers(mut self, header_map: http::header::HeaderMap) -> Request<B> {
        self.headers_mut().extend(header_map);
        self
    }

    /// Send the request to a service.
    ///
    /// If you enabled any decompression feature, the response body will be automatically decompressed.
    #[allow(unused_mut)]
    fn send<S, R>(self, mut client: S) -> impl Future<Output = crate::Result<S::Response>> + Send
    where
        B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<crate::error::BoxError>,
        S: tower_service::Service<Request<ClientBody>, Response = Response<R>> + Send + Sync,
        R: http_body::Body + Send + Sync + 'static,
        <S as tower_service::Service<Request<ClientBody>>>::Error: Into<crate::error::BoxError>,
        <S as tower_service::Service<Request<ClientBody>>>::Future: Send,
    {
        use http_body_util::BodyExt;
        let request = self.map(|b| BoxBody::new(b.map_err(|e| e.into())));
        client
            .call(request)
            .map_err(|e| crate::Error::SendRequest(e.into()))
    }
}

#[cfg(feature = "query")]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
fn build_query_uri<Q: Serialize + ?Sized>(uri: Uri, query: &Q) -> Result<Uri, BuildQueryError> {
    use std::str::FromStr;
    let new_query = serde_urlencoded::to_string(query)?;
    if new_query.is_empty() {
        return Ok(uri);
    }
    let mut uri_parts = uri.into_parts();
    let new_pq = if let Some(pq) = uri_parts.path_and_query {
        let mut new_pq_string = String::with_capacity(new_query.len() + pq.as_str().len() + 2);
        new_pq_string.push_str(pq.path());
        new_pq_string.push('?');
        if let Some(old_query) = pq.query() {
            new_pq_string.push_str(old_query);
            new_pq_string.push('&');
        }
        new_pq_string.push_str(&new_query);

        http::uri::PathAndQuery::from_str(&new_pq_string)?
    } else {
        http::uri::PathAndQuery::from_str(&new_query)?
    };
    uri_parts.path_and_query = Some(new_pq);
    let new_uri = Uri::from_parts(uri_parts)?;
    Ok(new_uri)
}

fn build_path_uri(uri: Uri, path: &str) -> Result<Uri, BuildPathError> {
    let mut parts = uri.into_parts();
    let Some(pq) = parts.path_and_query else {
        parts.path_and_query = Some(PathAndQuery::from_str(path)?);
        return Ok(Uri::from_parts(parts)?);
    };
    let query = pq.query();
    let pq = if let Some(query) = query {
        PathAndQuery::from_maybe_shared(format!("{path}?{query}"))?
    } else {
        PathAndQuery::from_str(path)?
    };
    parts.path_and_query = Some(pq);
    let uri = Uri::from_parts(parts)?;
    Ok(uri)
}
/*
    I copied and modified those tests from reqwest: https://github.com/seanmonstar/reqwest/blob/master/src/async_impl/request.rs
*/
#[cfg(test)]
mod tests {

    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn add_query_append() -> crate::Result<()> {
        let req = RequestBuilder::get("https://google.com/")?
            .query(&[("foo", "bar")])?
            .query(&[("qux", 3)])?
            .empty();

        assert_eq!(req.uri().query(), Some("foo=bar&qux=3"));
        Ok(())
    }

    #[test]
    fn add_query_append_same() -> crate::Result<()> {
        let req = RequestBuilder::get("https://google.com/")?
            .query(&[("foo", "a"), ("foo", "b")])?
            .empty();

        assert_eq!(req.uri().query(), Some("foo=a&foo=b"));
        Ok(())
    }

    #[test]
    fn add_query_struct() -> crate::Result<()> {
        #[derive(serde::Serialize)]
        struct Params {
            foo: String,
            qux: i32,
        }

        let params = Params {
            foo: "bar".into(),
            qux: 3,
        };
        let req = RequestBuilder::get("https://google.com/")?
            .query(&params)?
            .empty();

        assert_eq!(req.uri().query(), Some("foo=bar&qux=3"));
        Ok(())
    }

    #[test]
    fn add_query_map() -> crate::Result<()> {
        let mut params = BTreeMap::new();
        params.insert("foo", "bar");
        params.insert("qux", "three");

        let req = RequestBuilder::get("https://google.com/")?
            .query(&params)?
            .empty();
        assert_eq!(req.uri().query(), Some("foo=bar&qux=three"));
        Ok(())
    }

    #[test]
    fn test_replace_headers() -> crate::Result<()> {
        use http::HeaderMap;

        let mut headers = HeaderMap::new();
        headers.insert("foo", "bar".parse().unwrap());
        headers.append("foo", "baz".parse().unwrap());

        let req = RequestBuilder::get("https://hyper.rs")?
            .header("im-a", "keeper")?
            .header("foo", "pop me")?
            .headers(headers)
            .empty();

        assert_eq!(req.headers()["im-a"], "keeper");

        let foo = req.headers().get_all("foo").iter().collect::<Vec<_>>();
        assert_eq!(foo.len(), 2);
        assert_eq!(foo[0], "bar");
        assert_eq!(foo[1], "baz");
        Ok(())
    }

    #[test]
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    fn test_basic_auth_sensitive_header() -> crate::Result<()> {
        let some_url = "https://localhost/";

        let req = RequestBuilder::get(some_url)?
            .basic_auth("Aladdin", Some("open sesame"))
            .empty();

        assert_eq!(req.uri().to_string(), "https://localhost/");
        assert_eq!(
            req.headers()["authorization"],
            "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="
        );
        assert!(req.headers()["authorization"].is_sensitive());
        Ok(())
    }

    #[test]
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    fn test_bearer_auth_sensitive_header() -> crate::Result<()> {
        let some_url = "https://localhost/";

        let req = RequestBuilder::get(some_url)?
            .bearer_auth("Hold my bear")
            .empty();

        assert_eq!(req.uri().to_string(), "https://localhost/");
        assert_eq!(req.headers()["authorization"], "Bearer Hold my bear");
        assert!(req.headers()["authorization"].is_sensitive());
        Ok(())
    }

    #[test]
    fn test_explicit_sensitive_header() -> crate::Result<()> {
        let some_url = "https://localhost/";

        let mut header = http::HeaderValue::from_static("in plain sight");
        header.set_sensitive(true);

        let req = RequestBuilder::get(some_url)?.header("hiding", header)?;

        assert_eq!(req.uri.to_string(), "https://localhost/");
        assert_eq!(req.headers["hiding"], "in plain sight");
        assert!(req.headers["hiding"].is_sensitive());
        Ok(())
    }

    #[test]
    fn convert_from_http_request() -> crate::Result<()> {
        let req = Request::builder()
            .method("GET")
            .uri("http://localhost/")
            .header("User-Agent", "my-awesome-agent/1.0")
            .body("test test test")
            .unwrap();
        let test_data = b"test test test";
        assert_eq!(req.body().as_bytes(), &test_data[..]);
        let headers = req.headers();
        assert_eq!(headers.get("User-Agent").unwrap(), "my-awesome-agent/1.0");
        assert_eq!(req.method(), http::Method::GET);
        assert_eq!(req.uri().to_string(), "http://localhost/");
        Ok(())
    }

    #[test]
    fn set_http_request_version() -> crate::Result<()> {
        let req = Request::builder()
            .method("GET")
            .uri("http://localhost/")
            .header("User-Agent", "my-awesome-agent/1.0")
            .version(http::Version::HTTP_11)
            .body("test test test")
            .unwrap();
        let test_data = b"test test test";
        assert_eq!(req.body().as_bytes(), &test_data[..]);
        let headers = req.headers();
        assert_eq!(headers.get("User-Agent").unwrap(), "my-awesome-agent/1.0");
        assert_eq!(req.method(), http::Method::GET);
        assert_eq!(req.uri().to_string(), "http://localhost/");
        assert_eq!(req.version(), http::Version::HTTP_11);
        Ok(())
    }
}
