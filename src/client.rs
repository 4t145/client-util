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
use crate::error::BoxError as BodyError;
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
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

pub type ClientBody = BoxBody<Bytes, BodyError>;
