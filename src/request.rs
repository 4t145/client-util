#[cfg(feature = "multipart")]
mod multipart;
use bytes::Bytes;
use http::request::Builder;
use http::HeaderValue;
use http::Request;
use http::Response;
use http::{header::CONTENT_TYPE, Uri};
use http_body_util::{combinators::UnsyncBoxBody, Empty, Full};
#[cfg(feature = "multipart")]
pub use multipart::*;
use std::future::Future;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::body::{empty, full};
use crate::client::ClientBody;
use crate::client::MaybeAbort;
use crate::error::BodyError;
use crate::error::ClientError;

/// Extension trait for [`http::Request`].
pub trait RequestExt<B>: Sized {
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    fn json<T: Serialize + ?Sized>(self, body: &T) -> crate::Result<Request<Full<Bytes>>>;
    #[cfg(feature = "query")]
    #[cfg_attr(docsrs, doc(cfg(feature = "query")))]
    fn query<Q: Serialize + ?Sized>(self, query: &Q) -> crate::Result<Request<B>>;
    #[cfg(feature = "multipart")]
    #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
    fn multipart(
        self,
        form: multipart::Form,
    ) -> crate::Result<Request<UnsyncBoxBody<Bytes, BodyError>>>;
    #[cfg(feature = "form")]
    #[cfg_attr(docsrs, doc(cfg(feature = "form")))]
    fn form<T: Serialize + ?Sized>(self, form: &T) -> crate::Result<Request<Full<Bytes>>>;
    fn plain_text(self, body: impl Into<Bytes>) -> crate::Result<Request<Full<Bytes>>>;
    fn empty(self) -> crate::Result<Request<Empty<Bytes>>>;
    fn collect_into_bytes(self) -> impl Future<Output = crate::Result<Request<Full<Bytes>>>> + Send
    where
        B: http_body::Body<Data = Bytes> + Send + 'static,
        B::Error: std::error::Error + Send + Sync;
    fn with_version(self, version: http::Version) -> Request<B>;
    fn with_method(self, method: http::Method) -> Request<B>;
    fn with_header<K>(self, key: K, value: http::header::HeaderValue) -> Request<B>
    where
        K: http::header::IntoHeaderName;
    fn with_headers(self, header_map: http::header::HeaderMap) -> Request<B>;
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Request<B>
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
    fn bearer_auth<T>(self, token: T) -> Request<B>
    where
        T: std::fmt::Display,
    {
        let header_value = crate::util::bearer_auth(token);
        self.with_header(http::header::AUTHORIZATION, header_value)
    }

    fn send<S, R>(self, client: S) -> impl Future<Output = crate::Result<S::Response>> + Send
    where
        B: http_body::Body<Data = Bytes> + Send + 'static,
        B::Error: std::error::Error + Send + Sync,
        S: tower_service::Service<Request<ClientBody>, Response = Response<R>> + Send,
        R: http_body::Body,
        <S as tower_service::Service<Request<ClientBody>>>::Error:
            std::error::Error + Send + Sync + 'static,
        <S as tower_service::Service<Request<ClientBody>>>::Future: Send;

    #[cfg(feature = "rt-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
    /// Send the request to a service with a timeout layer.
    fn send_timeout<S, R>(
        self,
        client: S,
        timeout: std::time::Duration,
    ) -> impl Future<Output = crate::Result<Response<MaybeAbort<R>>>> + Send
    where
        B: http_body::Body<Data = Bytes> + Send + 'static,
        B::Error: std::error::Error + Send + Sync,
        S: tower_service::Service<Request<ClientBody>, Response = Response<R>> + Send,
        <S as tower_service::Service<Request<ClientBody>>>::Error:
            std::error::Error + Send + Sync + 'static,
        <S as tower_service::Service<Request<ClientBody>>>::Future: Send,
        R: http_body::Body,
        Self: Sized,
    {
        use tower::util::ServiceExt;
        self.send(tower_http::timeout::Timeout::new(
            client.map_response(|r| r.map(MaybeAbort::success)),
            timeout,
        ))
    }
}

/// Extension trait for [`http::request::Builder`].
pub trait RequestBuilderExt: Sized {
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    fn json<T: Serialize + ?Sized>(self, body: &T) -> crate::Result<Request<Full<Bytes>>>;
    #[cfg(feature = "query")]
    #[cfg_attr(docsrs, doc(cfg(feature = "query")))]
    fn query<Q: Serialize + ?Sized>(self, query: &Q) -> crate::Result<Self>;
    #[cfg(feature = "multipart")]
    #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
    fn multipart(self, form: multipart::Form) -> crate::Result<Request<crate::DynBody>>;
    #[cfg(feature = "form")]
    #[cfg_attr(docsrs, doc(cfg(feature = "form")))]
    fn form<T: Serialize + ?Sized>(self, form: &T) -> crate::Result<Request<Full<Bytes>>>;
    fn plain_text(self, body: impl Into<Bytes>) -> crate::Result<Request<Full<Bytes>>>;
    fn empty(self) -> crate::Result<Request<Empty<Bytes>>>;
    fn headers(self, header_map: http::header::HeaderMap) -> Self;
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: std::fmt::Display,
        P: std::fmt::Display,
        Self: Sized;
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    fn bearer_auth<T>(self, token: T) -> Self
    where
        T: std::fmt::Display;
}

impl RequestBuilderExt for Builder {
    /// Consumes the builder, setting the request body as JSON.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    fn json<T: Serialize + ?Sized>(self, body: &T) -> crate::Result<Request<Full<Bytes>>> {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.json(body)
    }

    /// Add query parameters to the request URI.
    #[cfg(feature = "query")]
    #[cfg_attr(docsrs, doc(cfg(feature = "query")))]
    fn query<Q: Serialize + ?Sized>(self, query: &Q) -> crate::Result<Self> {
        let new_uri = if let Some(uri) = self.uri_ref() {
            build_query_uri(uri.clone(), query)?
        } else {
            Uri::default()
        };
        Ok(self.uri(new_uri))
    }

    /// Consumes the builder, setting the request body as multipart form data.
    #[cfg(feature = "multipart")]
    #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
    fn multipart(self, form: multipart::Form) -> crate::Result<Request<crate::DynBody>> {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.multipart(form)
    }

    /// Consumes the builder, setting the request body as form data.
    #[cfg(feature = "form")]
    #[cfg_attr(docsrs, doc(cfg(feature = "form")))]
    fn form<T: Serialize + ?Sized>(self, form: &T) -> crate::Result<Request<Full<Bytes>>> {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.form(form)
    }

    /// Consumes the builder, setting the request body as plain text.
    fn plain_text(self, body: impl Into<Bytes>) -> crate::Result<Request<Full<Bytes>>> {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.plain_text(body)
    }

    /// Consumes the builder, setting the request body as empty.
    fn empty(self) -> crate::Result<Request<Empty<Bytes>>> {
        self.body(())
            .map_err(crate::Error::with_context("build request body"))?
            .empty()
    }

    /// Add basic authentication to the request.
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: std::fmt::Display,
        P: std::fmt::Display,
        Self: Sized,
    {
        let header_value = crate::util::basic_auth(username, password);
        self.header(http::header::AUTHORIZATION, header_value)
    }

    /// Add bearer authentication to the request.
    #[cfg(feature = "auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
    fn bearer_auth<T>(self, token: T) -> Self
    where
        T: std::fmt::Display,
    {
        let header_value = crate::util::bearer_auth(token);
        self.header(http::header::AUTHORIZATION, header_value)
    }

    /// Extend multiple request headers.
    fn headers(mut self, header_map: http::header::HeaderMap) -> Self {
        if let Some(headers) = self.headers_mut() {
            headers.extend(header_map);
        }
        self
    }
}

impl<B> RequestExt<B> for Request<B>
where
    B: Send,
{
    /// Set the request body as JSON.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    fn json<T: Serialize + ?Sized>(self, body: &T) -> crate::Result<Request<Full<Bytes>>> {
        let json_body =
            serde_json::to_vec(&body).map_err(crate::Error::with_context("serialize json body"))?;
        let (mut parts, _) = self.into_parts();
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );
        let request = Request::from_parts(parts, Full::new(Bytes::from(json_body)));
        Ok(request)
    }

    /// Add query parameters to the request URI.
    #[cfg(feature = "query")]
    #[cfg_attr(docsrs, doc(cfg(feature = "query")))]
    fn query<Q: Serialize + ?Sized>(self, query: &Q) -> crate::Result<Request<B>> {
        use http::uri::PathAndQuery;
        use std::str::FromStr;
        let new_query = serde_urlencoded::to_string(query)
            .map_err(crate::Error::with_context("serialize query string"))?;
        if new_query.is_empty() {
            return Ok(self);
        }
        let (mut parts, body) = self.into_parts();
        let mut uri_parts = parts.uri.into_parts();
        let new_uri = if let Some(pq) = uri_parts.path_and_query {
            let mut new_pq_string = String::with_capacity(new_query.len() + pq.as_str().len() + 2);
            new_pq_string.push_str(pq.path());
            new_pq_string.push('?');
            if let Some(old_query) = pq.query() {
                new_pq_string.push_str(old_query);
                new_pq_string.push('&');
            }
            new_pq_string.push_str(&new_query);
            let new_pq = PathAndQuery::from_str(&new_pq_string)
                .map_err(crate::Error::with_context("parse new path and query"))?;
            uri_parts.path_and_query = Some(new_pq);
            Uri::from_parts(uri_parts).map_err(crate::Error::with_context(
                "reconstruct uri with new path and query",
            ))?
        } else {
            Uri::builder()
                .path_and_query(new_query)
                .build()
                .map_err(crate::Error::with_context("build new uri"))?
        };
        parts.uri = new_uri;
        Ok(Request::from_parts(parts, body))
    }

    /// Set the request body as multipart form data.
    #[cfg(feature = "multipart")]
    #[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
    fn multipart(self, mut form: multipart::Form) -> crate::Result<Request<crate::DynBody>> {
        let (mut parts, _) = self.into_parts();
        let boundary = form.boundary();
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&format!(
                "{}; boundary={}",
                mime::MULTIPART_FORM_DATA,
                boundary
            ))
            .map_err(crate::Error::with_context("build content type header"))?,
        );
        if let Some(length) = form.compute_length() {
            parts.headers.insert(
                http::header::CONTENT_LENGTH,
                HeaderValue::from_str(&length.to_string())
                    .map_err(crate::Error::with_context("build content length header"))?,
            );
        }
        let body = form.stream();
        Ok(Request::from_parts(parts, body))
    }

    /// Set the request body as form data.
    #[cfg(feature = "form")]
    #[cfg_attr(docsrs, doc(cfg(feature = "form")))]
    fn form<T: Serialize + ?Sized>(self, form: &T) -> crate::Result<Request<Full<Bytes>>> {
        let (mut parts, _) = self.into_parts();
        let body = serde_urlencoded::to_string(form)
            .map_err(crate::Error::with_context("serialize form"))?;
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_WWW_FORM_URLENCODED.as_ref()),
        );
        Ok(Request::from_parts(parts, full(body)))
    }

    /// Set the request body as plain text.
    #[inline]
    fn plain_text(self, body: impl Into<Bytes>) -> crate::Result<Request<Full<Bytes>>> {
        let (mut parts, _) = self.into_parts();
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
        );
        Ok(Request::from_parts(parts, full(body)))
    }

    /// Set the request body as empty.
    #[inline]
    fn empty(self) -> crate::Result<Request<Empty<Bytes>>> {
        let (parts, _) = self.into_parts();
        Ok(Request::from_parts(parts, empty()))
    }

    /// Collect the request body stream into bytes. This is useful when you want to clone the request.
    async fn collect_into_bytes(self) -> crate::Result<Request<Full<Bytes>>>
    where
        B: http_body::Body<Data = Bytes> + Send + 'static,
        B::Error: std::error::Error + Send + Sync,
    {
        use http_body_util::BodyExt;
        let (parts, body) = self.into_parts();
        let body = body
            .collect()
            .await
            .map_err(|e| {
                crate::Error::new(
                    crate::ErrorKind::Body(BodyError(Box::new(e))),
                    "collect body stream",
                )
            })?
            .to_bytes();
        Ok(Request::from_parts(parts, full(body)))
    }

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
    async fn send<S, R>(self, mut client: S) -> crate::Result<S::Response>
    where
        B: http_body::Body<Data = Bytes> + Send + 'static,
        B::Error: std::error::Error + Send + Sync,
        S: tower_service::Service<Request<ClientBody>, Response = Response<R>> + Send,
        R: http_body::Body,
        <S as tower_service::Service<Request<ClientBody>>>::Error:
            std::error::Error + Send + Sync + 'static,
        <S as tower_service::Service<Request<ClientBody>>>::Future: Send,
    {
        use http_body_util::BodyExt;
        #[allow(unused_imports)]
        use tower_service::Service;
        #[cfg(all(
            any(
                feature = "decompression-deflate",
                feature = "decompression-gzip",
                feature = "decompression-br",
                feature = "decompression-zstd",
            ),
            feature = "rt-tokio"
        ))]
        let mut client = tower_http::decompression::Decompression::new(client);
        let request = self.map(|b| UnsyncBoxBody::new(b.map_err(|e| BodyError(Box::new(e)))));
        match client.call(request).await {
            #[cfg(all(
                any(
                    feature = "decompression-deflate",
                    feature = "decompression-gzip",
                    feature = "decompression-br",
                    feature = "decompression-zstd",
                ),
                feature = "rt-tokio"
            ))]
            Ok(response) => Ok(response.map(|b| b.into_inner())),
            #[cfg(not(all(
                any(
                    feature = "decompression-deflate",
                    feature = "decompression-gzip",
                    feature = "decompression-br",
                    feature = "decompression-zstd",
                ),
                feature = "rt-tokio"
            )))]
            Ok(response) => Ok(response),
            Err(e) => {
                let e = ClientError::from(e);
                Err(crate::Error::new(
                    crate::ErrorKind::Client(e),
                    "send request",
                ))
            }
        }
    }
}

#[cfg(feature = "query")]
fn build_query_uri<Q: Serialize + ?Sized>(uri: Uri, query: &Q) -> crate::Result<Uri> {
    use std::str::FromStr;
    let new_query = serde_urlencoded::to_string(query)
        .map_err(crate::Error::with_context("serialize query string"))?;
    if new_query.is_empty() {
        return Ok(uri);
    }
    let mut uri_parts = uri.into_parts();
    let new_uri = if let Some(pq) = uri_parts.path_and_query {
        let mut new_pq_string = String::with_capacity(new_query.len() + pq.as_str().len() + 2);
        new_pq_string.push_str(pq.path());
        new_pq_string.push('?');
        if let Some(old_query) = pq.query() {
            new_pq_string.push_str(old_query);
            new_pq_string.push('&');
        }
        new_pq_string.push_str(&new_query);
        let new_pq = http::uri::PathAndQuery::from_str(&new_pq_string)
            .map_err(crate::Error::with_context("parse new path and query"))?;
        uri_parts.path_and_query = Some(new_pq);
        Uri::from_parts(uri_parts).map_err(crate::Error::with_context(
            "reconstruct uri with new path and query",
        ))?
    } else {
        Uri::builder()
            .path_and_query(new_query)
            .build()
            .map_err(crate::Error::with_context("build new uri"))?
    };
    Ok(new_uri)
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
        let req = Request::get("https://google.com/")
            .query(&[("foo", "bar")])?
            .query(&[("qux", 3)])?
            .empty()?;

        assert_eq!(req.uri().query(), Some("foo=bar&qux=3"));
        Ok(())
    }

    #[test]
    fn add_query_append_same() -> crate::Result<()> {
        let req = Request::get("https://google.com/")
            .query(&[("foo", "a"), ("foo", "b")])?
            .empty()?;

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
        let req = Request::get("https://google.com/")
            .query(&params)?
            .empty()?;

        assert_eq!(req.uri().query(), Some("foo=bar&qux=3"));
        Ok(())
    }

    #[test]
    fn add_query_map() -> crate::Result<()> {
        let mut params = BTreeMap::new();
        params.insert("foo", "bar");
        params.insert("qux", "three");

        let req = Request::get("https://google.com/")
            .query(&params)?
            .empty()?;
        assert_eq!(req.uri().query(), Some("foo=bar&qux=three"));
        Ok(())
    }

    #[test]
    fn test_replace_headers() {
        use http::HeaderMap;

        let mut headers = HeaderMap::new();
        headers.insert("foo", "bar".parse().unwrap());
        headers.append("foo", "baz".parse().unwrap());

        let req = Request::get("https://hyper.rs")
            .header("im-a", "keeper")
            .header("foo", "pop me")
            .headers(headers)
            .empty()
            .expect("request build");

        assert_eq!(req.headers()["im-a"], "keeper");

        let foo = req.headers().get_all("foo").iter().collect::<Vec<_>>();
        assert_eq!(foo.len(), 2);
        assert_eq!(foo[0], "bar");
        assert_eq!(foo[1], "baz");
    }

    #[test]
    #[cfg(feature = "auth")]
    fn test_basic_auth_sensitive_header() {
        let some_url = "https://localhost/";

        let req = Request::get(some_url)
            .basic_auth("Aladdin", Some("open sesame"))
            .empty()
            .expect("request build");

        assert_eq!(req.uri().to_string(), "https://localhost/");
        assert_eq!(
            req.headers()["authorization"],
            "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="
        );
        assert!(req.headers()["authorization"].is_sensitive());
    }

    #[test]
    #[cfg(feature = "auth")]
    fn test_bearer_auth_sensitive_header() {
        let some_url = "https://localhost/";

        let req = Request::get(some_url)
            .bearer_auth("Hold my bear")
            .empty()
            .expect("request build");

        assert_eq!(req.uri().to_string(), "https://localhost/");
        assert_eq!(req.headers()["authorization"], "Bearer Hold my bear");
        assert!(req.headers()["authorization"].is_sensitive());
    }

    #[test]
    fn test_explicit_sensitive_header() {
        let some_url = "https://localhost/";

        let mut header = http::HeaderValue::from_static("in plain sight");
        header.set_sensitive(true);

        let req = Request::get(some_url)
            .header("hiding", header)
            .empty()
            .expect("request build");

        assert_eq!(req.uri().to_string(), "https://localhost/");
        assert_eq!(req.headers()["hiding"], "in plain sight");
        assert!(req.headers()["hiding"].is_sensitive());
    }

    #[test]
    fn convert_from_http_request() {
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
    }

    #[test]
    fn set_http_request_version() {
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
    }
}
