use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use bytes::Bytes;
use tower_http::BoxError;

pub type Body = BoxBody<Bytes, BoxError>;
pub fn full<B>(body: B) -> Body
where
    B: Into<Bytes>,
{
    let body = body.into();
    Full::new(body).map_err(Into::into).boxed()
}

pub fn empty() -> Body {
    Empty::new().map_err(Into::into).boxed()
}
