use crate::Body;

pub struct RequestContext {
    pub base_url: Option<http::Uri>,
    
}

pub trait Api {
    fn build_request(self, ctx: &RequestContext) -> crate::Result<http::Request<DynBody>>;
}

