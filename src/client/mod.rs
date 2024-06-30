//! This module contains the client implementations for the various HTTP clients.
//!
//! A client must be a (`Service`)[`tower_service::Service`]
//! that receives a [`http::Request<ClientBody>`] and returns a [`http::Response`].
//!
//! This crate provides a default client implementation using [`hyper`].
//!
//! However, you can use any service as a client, and add more layer upon it.
#[cfg(feature = "client-hyper")]
#[cfg_attr(docsrs, doc(cfg(feature = "client-hyper")))]
pub mod hyper;
use crate::error::BodyError;
use bytes::Bytes;
use http_body::Body;
use http_body_util::combinators::UnsyncBoxBody;
#[cfg(feature = "client-hyper")]
#[cfg_attr(docsrs, doc(cfg(feature = "client-hyper")))]
pub use hyper::*;

#[macro_export]
macro_rules! shared_client {
    ($v:vis $getter: ident: $maker: ident -> $ClientType: ty) => {
        $v fn $getter() -> $ClientType {
            static mut CLIENT: std::sync::OnceLock<$ClientType> = std::sync::OnceLock::new();
            unsafe {
                CLIENT.get_or_init($maker).clone()
            }
        }
    };
}

pub type ClientBody = UnsyncBoxBody<Bytes, BodyError>;
pin_project_lite::pin_project! {
    #[project = MaybeAbortProj]
    #[derive(Debug, Clone, Default)]
    /// Make a body default
    pub enum MaybeAbort<B> {
        #[default]
        Abort,
        Success {
            #[pin] body: B
        },
    }
}

impl<B> MaybeAbort<B> {
    pub fn success(body: B) -> Self {
        MaybeAbort::Success { body }
    }

    pub fn timeout() -> Self {
        MaybeAbort::Abort
    }
}

impl<B> Body for MaybeAbort<B>
where
    B: Body,
{
    type Data = B::Data;

    type Error = B::Error;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.project() {
            MaybeAbortProj::Abort => std::task::Poll::Ready(None),
            MaybeAbortProj::Success { body } => body.poll_frame(cx),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            MaybeAbort::Abort => true,
            MaybeAbort::Success { body } => body.is_end_stream(),
        }
    }

    fn size_hint(&self) -> http_body::SizeHint {
        match self {
            MaybeAbort::Abort => http_body::SizeHint::default(),
            MaybeAbort::Success { body } => body.size_hint(),
        }
    }
}
