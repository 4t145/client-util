#[cfg(feature = "multipart")]
mod multipart;
use bytes::Bytes;
use http::request::Builder;
use http::Request;
use http::Response;
use http::{header::CONTENT_TYPE, Uri};
use http::{uri::Scheme, HeaderValue};
#[cfg(feature = "multipart")]
pub use multipart::*;
use std::future::Future;

#[cfg(feature = "serde")]
use serde::Serialize;

use crate::body::{empty, full};

pub trait TryIntoRequest {
    type Error: std::error::Error + Send + 'static;
    type Body: http_body::Body + Send;
    fn try_into_request(self) -> Result<Request<Self::Body>, Self::Error>;
}

pub trait RequestModifier<T, U> {
    fn modify(self, req: Request<T>) -> crate::Result<Request<U>>;
}

pub trait RequestExt<B> {
    #[cfg(feature = "json")]
    fn json<T: Serialize + ?Sized>(self, body: &T) -> crate::Result<Request<crate::Body>>;
    #[cfg(feature = "query")]
    fn query<Q: Serialize + ?Sized>(self, query: &Q) -> crate::Result<Request<B>>;
    #[cfg(feature = "multipart")]
    fn multipart(self, form: multipart::Form) -> crate::Result<Request<crate::Body>>;
    #[cfg(feature = "form")]
    fn form<T: Serialize + ?Sized>(self, form: &T) -> crate::Result<Request<crate::Body>>;
    fn plain_text(self, body: impl Into<Bytes>) -> crate::Result<Request<crate::Body>>;
    fn empty(self) -> crate::Result<Request<crate::Body>>;
    fn with_version(self, version: http::Version) -> crate::Result<Request<B>>;
    fn with_method(self, method: http::Method) -> crate::Result<Request<B>>;
    fn with_schema(self, schema: http::uri::Scheme) -> crate::Result<Request<B>>;
    fn with_header<K>(self, key: K, value: http::header::HeaderValue) -> crate::Result<Request<B>>
    where
        K: http::header::IntoHeaderName;
    #[cfg(feature = "basic_auth")]
    fn with_basic_auth<U, P>(self, username: U, password: Option<P>) -> crate::Result<Request<B>>
    where
        U: std::fmt::Display,
        P: std::fmt::Display,
        Self: Sized,
    {
        let header_value = crate::util::basic_auth(username, password);
        self.with_header(http::header::AUTHORIZATION, header_value)
    }

    fn apply<M, U>(self, modifier: M) -> crate::Result<Request<U>>
    where
        M: RequestModifier<B, U>;

    fn send<S, R>(self, client: S) -> impl Future<Output = crate::Result<S::Response>> + Send
    where
        S: tower_service::Service<Request<B>, Response = Response<R>> + Send,
        S::Error: Into<crate::ErrorKind>,
        S::Future: Send;

    fn send_timeout<S, R>(
        self,
        client: S,
        timeout: std::time::Duration,
    ) -> impl Future<Output = crate::Result<S::Response>> + Send
    where
        S: tower_service::Service<Request<B>, Response = Response<R>> + Send,
        S::Error: Into<crate::ErrorKind>,
        S::Future: Send,
        R: Default,
        Self: Sized,
    {
        self.send(tower_http::timeout::Timeout::new(client, timeout))
    }
}

impl RequestExt<()> for Builder {
    #[cfg(feature = "json")]
    fn json<T: Serialize + ?Sized>(self, body: &T) -> crate::Result<Request<crate::Body>> {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.json(body)
    }

    #[cfg(feature = "query")]
    fn query<Q: Serialize + ?Sized>(self, query: &Q) -> crate::Result<Request<()>> {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.query(query)
    }

    #[cfg(feature = "multipart")]
    fn multipart(self, form: multipart::Form) -> crate::Result<Request<crate::Body>> {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.multipart(form)
    }

    #[cfg(feature = "form")]
    fn form<T: Serialize + ?Sized>(self, form: &T) -> crate::Result<Request<crate::Body>> {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.form(form)
    }

    fn plain_text(self, body: impl Into<Bytes>) -> crate::Result<Request<crate::Body>> {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.plain_text(body)
    }

    fn empty(self) -> crate::Result<Request<crate::Body>> {
        self.body(())
            .map_err(crate::Error::with_context("build request body"))?
            .empty()
    }
    fn with_version(self, version: http::Version) -> crate::Result<Request<()>> {
        self.version(version)
            .body(())
            .map_err(crate::Error::with_context("build request body"))
    }

    fn with_method(self, method: http::Method) -> crate::Result<Request<()>> {
        self.method(method)
            .body(())
            .map_err(crate::Error::with_context("build request body"))
    }

    fn with_schema(self, schema: http::uri::Scheme) -> crate::Result<Request<()>> {
        self.body(())
            .map_err(crate::Error::with_context("build request body"))?
            .with_schema(schema)
    }

    fn with_header<K>(self, key: K, value: http::header::HeaderValue) -> crate::Result<Request<()>>
    where
        K: http::header::IntoHeaderName,
    {
        self.body(())
            .map_err(crate::Error::with_context("build request body"))?
            .with_header(key, value)
    }

    fn apply<M, U>(self, modifier: M) -> crate::Result<Request<U>>
    where
        M: RequestModifier<(), U>,
    {
        modifier.modify(
            self.body(())
                .map_err(crate::Error::with_context("build request body"))?,
        )
    }

    async fn send<S, R>(self, client: S) -> crate::Result<S::Response>
    where
        S: tower_service::Service<Request<()>, Response = Response<R>> + Send,
        S::Error: Into<crate::ErrorKind>,
        S::Future: Send,
    {
        let req = self
            .body(())
            .map_err(crate::Error::with_context("build request body"))?;
        req.send(client).await
    }
}

impl<B> RequestExt<B> for Request<B>
where
    B: Send,
{
    #[cfg(feature = "json")]
    fn json<T: Serialize + ?Sized>(self, body: &T) -> crate::Result<Request<crate::Body>> {
        let json_body =
            serde_json::to_vec(&body).map_err(crate::Error::with_context("serialize json body"))?;
        let (mut parts, _) = self.into_parts();
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );
        let request = Request::from_parts(parts, full(json_body));
        Ok(request)
    }

    #[cfg(feature = "query")]
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

    #[cfg(feature = "multipart")]
    fn multipart(self, mut form: multipart::Form) -> crate::Result<Request<crate::Body>> {
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

    #[cfg(feature = "form")]
    fn form<T: Serialize + ?Sized>(self, form: &T) -> crate::Result<Request<crate::Body>> {
        let (mut parts, _) = self.into_parts();
        let body = serde_urlencoded::to_string(form)
            .map_err(crate::Error::with_context("serialize form"))?;
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_WWW_FORM_URLENCODED.as_ref()),
        );
        Ok(Request::from_parts(parts, full(body)))
    }

    fn plain_text(self, body: impl Into<Bytes>) -> crate::Result<Request<crate::Body>> {
        let (mut parts, _) = self.into_parts();
        parts.headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
        );
        Ok(Request::from_parts(parts, full(body)))
    }
    fn empty(self) -> crate::Result<Request<crate::Body>> {
        let (parts, _) = self.into_parts();
        Ok(Request::from_parts(parts, empty()))
    }

    fn with_version(mut self, version: http::Version) -> crate::Result<Request<B>> {
        *self.version_mut() = version;
        Ok(self)
    }

    fn with_method(mut self, method: http::Method) -> crate::Result<Request<B>> {
        *self.method_mut() = method;
        Ok(self)
    }

    fn with_schema(self, schema: Scheme) -> crate::Result<Request<B>> {
        let (mut parts, body) = self.into_parts();
        let mut uri_parts = parts.uri.into_parts();
        uri_parts.scheme = Some(schema);
        parts.uri = Uri::from_parts(uri_parts).map_err(crate::Error::with_context(
            "reconstruct uri with new schema",
        ))?;
        Ok(Request::from_parts(parts, body))
    }

    fn with_header<K>(
        mut self,
        key: K,
        value: http::header::HeaderValue,
    ) -> crate::Result<Request<B>>
    where
        K: http::header::IntoHeaderName,
    {
        self.headers_mut().insert(key, value);
        Ok(self)
    }

    fn apply<M, U>(self, modifier: M) -> crate::Result<Request<U>>
    where
        M: RequestModifier<B, U>,
    {
        modifier.modify(self)
    }

    async fn send<S, R>(self, mut client: S) -> crate::Result<S::Response>
    where
        S: tower_service::Service<Request<B>, Response = Response<R>> + Send,
        S::Error: Into<crate::ErrorKind>,
    {
        match client.call(self).await {
            Ok(response) => Ok(response),
            Err(e) => {
                let kind = e.into();
                Err(crate::Error::new(kind, "send request"))
            }
        }
    }
}