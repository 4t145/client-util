#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    future_incompatible,
    nonstandard_style
)]
pub mod body;
pub mod client;
pub mod error;
pub mod request;
pub mod response;
pub mod api;
mod util;

pub use body::{empty, full, Body};
pub use error::{Error, Result};

// re-export
pub use http;
pub use http_body;
pub use http_body_util;

pub mod prelude {
    pub use crate::body::*;
    pub use crate::client::*;
    pub use crate::error::*;
    pub use crate::request::*;
    pub use crate::response::*;
}
