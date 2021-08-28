use axum::{body, http, response::IntoResponse, BoxError};

use crate::error::Error;

#[derive(Debug)]
#[non_exhaustive]
pub struct InvalidMsgPackBody(Error);

impl InvalidMsgPackBody {
    pub(crate) fn from_err<E>(err: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self(Error::new(err))
    }
}

#[allow(deprecated)]
impl IntoResponse for InvalidMsgPackBody {
    type Body = body::Full<body::Bytes>;
    type BodyError = std::convert::Infallible;

    fn into_response(self) -> http::Response<Self::Body> {
        let mut res = http::Response::new(body::Full::from(format!(
            concat!("Failed to parse the request body as MsgPack", ": {}"),
            self.0
        )));
        *res.status_mut() = http::StatusCode::BAD_REQUEST;
        res
    }
}

impl std::fmt::Display for InvalidMsgPackBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "Failed to parse the request body as MsgPack")
    }
}

impl std::error::Error for InvalidMsgPackBody {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct MissingMsgPackContentType;

#[allow(deprecated)]
impl IntoResponse for MissingMsgPackContentType {
    type Body = body::Full<body::Bytes>;
    type BodyError = std::convert::Infallible;

    fn into_response(self) -> http::Response<Self::Body> {
        let mut res = http::Response::new(body::Full::from(
            "Expected request with `Content-Type: application/msgpack`",
        ));
        *res.status_mut() = http::StatusCode::BAD_REQUEST;
        res
    }
}

impl std::fmt::Display for MissingMsgPackContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            "Expected request with `Content-Type: application/msgpack`"
        )
    }
}

impl std::error::Error for MissingMsgPackContentType {}

#[derive(Debug)]
#[non_exhaustive]
pub struct BodyAlreadyExtracted;

#[allow(deprecated)]
impl IntoResponse for BodyAlreadyExtracted {
    type Body = body::Full<body::Bytes>;
    type BodyError = std::convert::Infallible;

    fn into_response(self) -> http::Response<Self::Body> {
        let mut res = http::Response::new(body::Full::from(
            "Cannot have two request body extractors for a single handler",
        ));
        *res.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
        res
    }
}

impl std::fmt::Display for BodyAlreadyExtracted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            "Cannot have two request body extractors for a single handler"
        )
    }
}

impl std::error::Error for BodyAlreadyExtracted {}

#[derive(Debug)]
#[non_exhaustive]
pub struct HeadersAlreadyExtracted;

#[allow(deprecated)]
impl IntoResponse for HeadersAlreadyExtracted {
    type Body = body::Full<body::Bytes>;
    type BodyError = std::convert::Infallible;

    fn into_response(self) -> http::Response<Self::Body> {
        let mut res = http::Response::new(body::Full::from("Headers taken by other extractor"));
        *res.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
        res
    }
}

impl std::fmt::Display for HeadersAlreadyExtracted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "Headers taken by other extractor")
    }
}

impl std::error::Error for HeadersAlreadyExtracted {}

#[derive(Debug)]
#[non_exhaustive]
pub enum MsgPackRejection {
    InvalidMsgPackBody(InvalidMsgPackBody),
    MissingMsgPackContentType(MissingMsgPackContentType),
    BodyAlreadyExtracted(BodyAlreadyExtracted),
    HeadersAlreadyExtracted(HeadersAlreadyExtracted),
}

impl IntoResponse for MsgPackRejection {
    type Body = body::Full<body::Bytes>;
    type BodyError = std::convert::Infallible;

    fn into_response(self) -> http::Response<Self::Body> {
        match self {
            Self::InvalidMsgPackBody(inner) => inner.into_response(),
            Self::MissingMsgPackContentType(inner) => inner.into_response(),
            Self::BodyAlreadyExtracted(inner) => inner.into_response(),
            Self::HeadersAlreadyExtracted(inner) => inner.into_response(),
        }
    }
}

#[allow(deprecated)]
impl From<InvalidMsgPackBody> for MsgPackRejection {
    fn from(inner: InvalidMsgPackBody) -> Self {
        Self::InvalidMsgPackBody(inner)
    }
}

#[allow(deprecated)]
impl From<MissingMsgPackContentType> for MsgPackRejection {
    fn from(inner: MissingMsgPackContentType) -> Self {
        Self::MissingMsgPackContentType(inner)
    }
}

#[allow(deprecated)]
impl From<BodyAlreadyExtracted> for MsgPackRejection {
    fn from(inner: BodyAlreadyExtracted) -> Self {
        Self::BodyAlreadyExtracted(inner)
    }
}

#[allow(deprecated)]
impl From<HeadersAlreadyExtracted> for MsgPackRejection {
    fn from(inner: HeadersAlreadyExtracted) -> Self {
        Self::HeadersAlreadyExtracted(inner)
    }
}

impl std::fmt::Display for MsgPackRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMsgPackBody(inner) => write!(f, "{}", inner),
            Self::MissingMsgPackContentType(inner) => write!(f, "{}", inner),
            Self::BodyAlreadyExtracted(inner) => write!(f, "{}", inner),
            Self::HeadersAlreadyExtracted(inner) => write!(f, "{}", inner),
        }
    }
}

impl std::error::Error for MsgPackRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidMsgPackBody(inner) => Some(inner),
            Self::MissingMsgPackContentType(inner) => Some(inner),
            Self::BodyAlreadyExtracted(inner) => Some(inner),
            Self::HeadersAlreadyExtracted(inner) => Some(inner),
        }
    }
}
