use std::{
    convert::Infallible,
    ops::{Deref, DerefMut},
};

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    response::IntoResponse,
    BoxError,
};
use axum::{
    body,
    http::{header::HeaderValue, StatusCode},
};

use hyper::{body::Buf, header, Response};
use rejection::{BodyAlreadyExtracted, HeadersAlreadyExtracted, MsgPackRejection};
use serde::{de::DeserializeOwned, Serialize};

use crate::rejection::{InvalidMsgPackBody, MissingMsgPackContentType};

mod error;
mod rejection;

#[derive(Debug, Clone, Copy, Default)]
pub struct MsgPack<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for MsgPack<T>
where
    T: DeserializeOwned,
    B: axum::body::HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = MsgPackRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        if has_content_type(req, "application/msgpack")? {
            let body = take_body(req)?;

            let buf = hyper::body::aggregate(body)
                .await
                .map_err(InvalidMsgPackBody::from_err)?;

            let value =
                rmp_serde::decode::from_read(buf.reader()).map_err(InvalidMsgPackBody::from_err)?;
            Ok(MsgPack(value))
        } else {
            Err(MissingMsgPackContentType.into())
        }
    }
}

impl<T> Deref for MsgPack<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MsgPack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for MsgPack<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for MsgPack<T>
where
    T: Serialize,
{
    type Body = body::Full<body::Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let bytes = match rmp_serde::encode::to_vec_named(&self.0) {
            Ok(res) => res,
            Err(err) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(body::Full::from(err.to_string()))
                    .unwrap();
            }
        };

        let mut res = Response::new(body::Full::from(bytes));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/msgpack"),
        );
        res
    }
}

pub(crate) fn has_content_type<B>(
    req: &RequestParts<B>,
    expected_content_type: &str,
) -> Result<bool, HeadersAlreadyExtracted> {
    let content_type = if let Some(content_type) = req
        .headers()
        .ok_or(HeadersAlreadyExtracted)?
        .get(header::CONTENT_TYPE)
    {
        content_type
    } else {
        return Ok(false);
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return Ok(false);
    };

    Ok(content_type.starts_with(expected_content_type))
}

pub(crate) fn take_body<B>(req: &mut RequestParts<B>) -> Result<B, BodyAlreadyExtracted> {
    req.take_body().ok_or(BodyAlreadyExtracted)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
