use axum::extract::RequestParts;
use hyper::http::header;
use rejection::{BodyAlreadyExtracted, HeadersAlreadyExtracted};

mod error;
mod rejection;

#[cfg(test)]
mod test_helpers;

mod msgpack;
mod msgpackraw;

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
    use axum::{routing::post, Json, Router};
    use serde::Deserialize;
    use crate::{ test_helpers::*};

    #[tokio::test]
    async fn deserialize_body() {
        #[derive(Debug, Deserialize)]
        struct Input {
            foo: String,
        }
        let app = Router::new().route("/", post(|input: Json<Input>| async { input.0.foo }));

        let client = TestClient::new(app);
      

        // let app = Router::new().route("/", post(|input: Json<Input>| async { input.0.foo }));

        // let client = TestClient::new(app);
        // let res = client.post("/").json(&json!({ "foo": "bar" })).send().await;
        // let body = res.text().await;

        // assert_eq!(body, "bar");
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
