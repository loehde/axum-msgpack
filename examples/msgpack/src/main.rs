
use axum::{response::Html, routing::get, Router};
use std::net::SocketAddr;
use axum_msgpack::MsgPack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    pub name: String,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/", get(get_handler).post(post_handler));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_handler() -> MsgPack<User> {
    let user = User {
        name: "user name".to_string(),
        data: vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
    };
    MsgPack(user)
}

async fn post_handler(MsgPack(user): MsgPack<User>) -> Html<String> {
    let string = format!("<h1>{:?}</h1>", user.name);
    Html(string)
}