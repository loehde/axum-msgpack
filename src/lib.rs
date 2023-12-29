#![forbid(unsafe_code)]

use crate::rejection::{InvalidMsgPackBody, MissingMsgPackContentType};
use axum::{
    body::{Bytes, Body},
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
    http::{header::HeaderValue, StatusCode},
    async_trait,
};
use hyper::header;
use rejection::MsgPackRejection;
use serde::{de::DeserializeOwned, Serialize};
use std::ops::{Deref, DerefMut};

mod error;
mod rejection;

/// MessagePack Extractor / Response.
///
/// When used as an extractor, it can deserialize request bodies into some type that
/// implements [`serde::Deserialize`]. If the request body cannot be parsed, or value of the
/// `Content-Type` header does not match any of the `application/msgpack`, `application/x-msgpack`
/// or `application/*+msgpack` it will reject the request and return a `400 Bad Request` response.
///
/// # Extractor example
///
/// ```no_run
/// use axum::{
///     routing::post,
///     Router,
/// };
/// use axum_msgpack::MsgPack;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     email: String,
///     password: String,
/// }
///
/// async fn create_user(MsgPack(payload): MsgPack<CreateUser>) {
///     // payload is a `CreateUser`
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// # async {
/// #   axum::serve(tokio::net::TcpListener::bind(&"").await.unwrap(), app.into_make_service()).await.unwrap();
/// # };
/// ```
///
/// When used as a response, it can serialize any type that implements [`serde::Serialize`] to
/// `MsgPack`, and will automatically set `Content-Type: application/msgpack` header.
///
/// # Response example
///
/// ```no_run
/// use axum::{
///     extract::Path,
///     routing::get,
///     Router,
/// };
/// use axum_msgpack::MsgPack;
/// use serde::Serialize;
/// use uuid::Uuid;
///
/// #[derive(Serialize)]
/// struct User {
///     id: Uuid,
///     username: String,
/// }
///
/// async fn get_user(Path(user_id) : Path<Uuid>) -> MsgPack<User> {
///     let user = find_user(user_id).await;
///     MsgPack(user)
/// }
///
/// async fn find_user(user_id: Uuid) -> User {
///     // ...
///     # unimplemented!()
/// }
///
/// let app = Router::new().route("/users/:id", get(get_user));
/// # async {
/// #   axum::serve(tokio::net::TcpListener::bind(&"").await.unwrap(), app.into_make_service()).await.unwrap();
/// # };
/// # mod uuid {
/// #   use serde::{Serialize, Deserialize};
/// #   #[derive(Serialize, Deserialize)]
/// #   pub struct Uuid;
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MsgPack<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for MsgPack<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = MsgPackRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if !message_pack_content_type(&req) {
            return Err(MissingMsgPackContentType.into())
        }
        let bytes = Bytes::from_request(req, state).await?;
        let value = rmp_serde::from_slice(&bytes).map_err(InvalidMsgPackBody::from_err)?;
        Ok(MsgPack(value))
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
    fn into_response(self) -> Response {
        let bytes = match rmp_serde::encode::to_vec_named(&self.0) {
            Ok(res) => res,
            Err(err) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::new(err.to_string()))
                    .unwrap();
            }
        };

        let mut res = bytes.into_response();

        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/msgpack"),
        );
        res
    }
}

/// MessagePack Extractor / Response.
///
/// When used as an extractor, it can deserialize request bodies into some type that
/// implements [`serde::Deserialize`]. If the request body cannot be parsed, or value of the
/// `Content-Type` header does not match any of the `application/msgpack`, `application/x-msgpack`
/// or `application/*+msgpack` it will reject the request and return a `400 Bad Request` response.
///
/// # Extractor example
///
/// ```no_run
/// use axum::{
///     routing::post,
///     Router,
/// };
/// use axum_msgpack::MsgPackRaw;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     email: String,
///     password: String,
/// }
///
/// async fn create_user(MsgPackRaw(payload): MsgPackRaw<CreateUser>) {
///     // payload is a `CreateUser`
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// # async {
/// #   axum::serve(tokio::net::TcpListener::bind(&"").await.unwrap(), app.into_make_service()).await.unwrap();
/// # };
/// ```
///
/// When used as a response, it can serialize any type that implements [`serde::Serialize`] to
/// `MsgPackRaw`, and will automatically set `Content-Type: application/msgpack` header.
///
/// # Response example
///
/// ```no_run
/// use axum::{
///     extract::Path,
///     routing::get,
///     Router,
/// };
/// use axum_msgpack::MsgPackRaw;
/// use serde::Serialize;
/// use uuid::Uuid;
///
/// #[derive(Serialize)]
/// struct User {
///     id: Uuid,
///     username: String,
/// }
///
/// async fn get_user(Path(user_id) : Path<Uuid>) -> MsgPackRaw<User> {
///     let user = find_user(user_id).await;
///     MsgPackRaw(user)
/// }
///
/// async fn find_user(user_id: Uuid) -> User {
///     // ...
///     # unimplemented!()
/// }
///
/// let app = Router::new().route("/users/:id", get(get_user));
/// # async {
/// #   axum::serve(tokio::net::TcpListener::bind(&"").await.unwrap(), app.into_make_service()).await.unwrap();
/// # };
/// # mod uuid {
/// #   use serde::{Serialize, Deserialize};
/// #   #[derive(Serialize, Deserialize)]
/// #   pub struct Uuid;
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MsgPackRaw<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for MsgPackRaw<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = MsgPackRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if !message_pack_content_type(&req) {
            return Err(MissingMsgPackContentType.into())
        } 
        let bytes = Bytes::from_request(req, state).await?;
        let value = rmp_serde::from_slice(&bytes).map_err(InvalidMsgPackBody::from_err)?;
        Ok(MsgPackRaw(value))
    }
}

impl<T> Deref for MsgPackRaw<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MsgPackRaw<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for MsgPackRaw<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for MsgPackRaw<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let bytes = match rmp_serde::encode::to_vec(&self.0) {
            Ok(res) => res,
            Err(err) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(Body::new(err.to_string()))
                    .unwrap();
            }
        };

        let mut res = bytes.into_response();

        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/msgpack"),
        );
        res
    }
}

fn message_pack_content_type<B>(req: &Request<B>) -> bool {
    let Some(content_type) = req.headers().get(header::CONTENT_TYPE) else {
        return false;
    };
    let  Ok(content_type) = content_type.to_str() else {
        return false;
    };
    let Ok(mime) = content_type.parse::<mime::Mime>() else {
        return false;
    };

    let is_message_pack = mime.type_() == "application"
        && (["msgpack", "x-msgpack"]
            .iter()
            .any(|subtype| *subtype == mime.subtype())
            || mime.suffix().map_or(false, |suffix| suffix == "msgpack"));

    is_message_pack
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::FromRequest,
        http::HeaderValue,
        response::IntoResponse,
    };
    use futures_util::StreamExt;

    use crate::{MsgPack, MsgPackRaw, MsgPackRejection};
    use hyper::{header, Request};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Input {
        foo: String,
    }

    fn into_request<T: Serialize>(value: &T) -> Request<Body> {
        let serialized =
            rmp_serde::encode::to_vec_named(&value).expect("Failed to serialize test struct");

        let body = Body::from(serialized);
        Request::new(body)
    }

    fn into_request_raw<T: Serialize>(value: &T) -> Request<Body> {
        let serialized =
            rmp_serde::encode::to_vec(&value).expect("Failed to serialize test struct");

        let body = Body::from(serialized);
        Request::new(body)
    }

    #[tokio::test]
    async fn serializes_named() {
        let input = Input { foo: "bar".into() };
        let serialized = rmp_serde::encode::to_vec_named(&input);
        assert!(serialized.is_ok());
        let serialized = serialized.unwrap();

        let body = MsgPack(input).into_response().into_body();
        let bytes = to_bytes(body).await;

        assert_eq!(serialized, bytes);
    }
    
    #[tokio::test]
    async fn deserializes_named() {
        let input = Input { foo: "bar".into() };
        let mut request = into_request(&input);

        request.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/msgpack"),
        );
        
        let outcome =
            <MsgPack<Input> as FromRequest<_, _>>::from_request(request, &||{}).await;
        
        let outcome = outcome.unwrap();
        assert_eq!(input, outcome.0);
    }

    #[tokio::test]
    async fn serializes_raw() {
        let input = Input { foo: "bar".into() };
        let serialized = rmp_serde::encode::to_vec(&input);
        assert!(serialized.is_ok());
        let serialized = serialized.unwrap();

        let body = MsgPackRaw(input).into_response().into_body();
        let bytes = to_bytes(body).await;

        assert_eq!(serialized, bytes);
    }

    #[tokio::test]
    async fn deserializes_raw() {
        let input = Input { foo: "bar".into() };
        let mut request = into_request_raw(&input);

        request.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/msgpack"),
        );

        let outcome =
            <MsgPackRaw<Input> as FromRequest<_, _>>::from_request(request, &||{})
                .await;
        
        let outcome = outcome.unwrap();
        assert_eq!(input, outcome.0);
    }

    #[tokio::test]
    async fn supported_content_type() {
        let input = Input { foo: "bar".into() };
        let mut request = into_request(&input);
        request.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/msgpack"),
        );

        let outcome =
            <MsgPack<Input> as FromRequest<_, _>>::from_request(request, &||{}).await;
        assert!(outcome.is_ok());

        let mut request = into_request(&input);
        request.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/cloudevents+msgpack"),
        );

        let outcome =
            <MsgPack<Input> as FromRequest<_, _>>::from_request(request, &||{}).await;
        assert!(outcome.is_ok());

        let mut request = into_request(&input);
        request.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-msgpack"),
        );

        let outcome =
            <MsgPack<Input> as FromRequest<_, _>>::from_request(request, &||{}).await;
        assert!(outcome.is_ok());

        let request = into_request(&input);
        let outcome =
            <MsgPack<Input> as FromRequest<_, _>>::from_request(request, &||{}).await;

        match outcome {
            Err(MsgPackRejection::MissingMsgPackContentType(_)) => {}
            other => unreachable!(
                "Expected missing MsgPack content type rejection, got: {:?}",
                other
            ),
        }
    }

    async fn to_bytes(body: Body) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut stream = body.into_data_stream();

        while let Some(bytes) = stream.next().await {
            buffer.extend(bytes.unwrap().into_iter());
        }

        buffer
    }
}
