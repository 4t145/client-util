pub mod body;
pub mod client;
pub mod error;
pub mod request;
pub mod response;
mod util;

pub use error::{Error, ErrorKind, Result};
pub use body::{Body, full, empty};

// re-export
pub use http;

pub mod prelude {
    pub use crate::body::*;
    pub use crate::error::*;
    pub use crate::client::*;
    pub use crate::request::*;
    pub use crate::response::*;
}