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
use serde::{de::DeserializeOwned, Serialize};

use crate::{util::has_content_type, rejection::{InvalidMsgPackBody, MissingMsgPackContentType, MsgPackRejection}, util::take_body};

/// MsgPack with named fields
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


#[cfg(test)]
mod tests {
    use axum::{routing::post, Json, Router};
    use serde::{Deserialize, Serialize};
    use tokio::test;
    use crate::{MsgPack, MsgPackRaw, test_helpers::*};

    #[derive(Debug, Serialize, Deserialize)]
    struct Input {
        foo: String,
    }

    #[test]
    async fn deserialize_body() {

        let app = Router::new().route("/", post(|input: MsgPack<Input>| async { MsgPack(Input { foo: "pass".to_string()}) }));

        let client = TestClient::new(app)
                .post("/")
                .header("content-type", "application/msgpack")
                .msgpack(&Input {foo: "bar".to_string()})
                .send()
                .await;

        let rt: Input = client.msgpack().await;
        println!("{:?}", rt);
    }

    #[test]
    async fn deserializef_body() {

        let app = Router::new().route("/", post(|input: MsgPack<Input>| async { MsgPackRaw(Input { foo: "pass".to_string()}) }));

        let client = TestClient::new(app)
                .post("/")
                .header("content-type", "application/msgpack")
                .msgpack(&Input {foo: "bar".to_string()})
                .send()
                .await;

        let rt: Input = client.msgpack().await;
        println!("{:?}", rt);
    }
}

