#![forbid(unsafe_code)]
use axum::extract::RequestParts;
use hyper::http::header;
use rejection::{BodyAlreadyExtracted, HeadersAlreadyExtracted};

mod error;
mod rejection;
mod util;

#[cfg(test)]
mod test_helpers;

mod msgpack;
mod msgpack_raw;

pub use msgpack::MsgPack;
pub use msgpack_raw::MsgPackRaw;
