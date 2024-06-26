use http::header::CONTENT_TYPE;
pub use http::request::Builder;
pub use http::Request;
use hyper::body::Body;
use serde::Serialize;

use crate::body::json::Json;
use crate::header::APPLICATION_JSON;

pub trait RequestBuilderExt {
    fn json<B: Serialize>(self, body: B) -> Request<Json<B>>;
}

impl RequestBuilderExt for Builder {
    fn json<B: Serialize>(mut self, body: B) -> Request<Json<B>> {
        self = self.header(CONTENT_TYPE, APPLICATION_JSON);
        self.body(Json::new(body)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::StatusCode;
    use serde_json::json;

    #[test]
    fn test_request_builder_ext_json() {
        let req = Request::builder()
        .json(json!({"hello": "world"}));

        assert_eq!(req.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON);
    }
}
