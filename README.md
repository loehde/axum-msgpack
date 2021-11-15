# axum-msgpack

`axum-msgpack` add MessagePack features to axum.

[![Documentation](https://docs.rs/axum-msgpack/badge.svg)](https://docs.rs/axum-msgpack)
[![Crates.io](https://img.shields.io/crates/v/axum-msgpack)](https://crates.io/crates/axum-msgpack)

More information about this crate can be found in the [crate documentation][docs].

https://serde.rs/
https://msgpack.org/

## Usage example

```rust
use axum_msgpack::MsgPack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    pub name: String,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>
}

// axum handler | MsgPack
async fn get_handler() -> MsgPack<User> {
    let user = User {
        name: "steve".to_string(),
        data: vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
    };
    MsgPack(user)
}

// axum handler | MsgPack
async fn post_handler(MsgPack(user): MsgPack<User>) -> Html<String> {
    let string = format!("<h1>{:?}</h1>", user.name);
    Html(string)
}

// axum handler | MsgPackRaw
async fn get_handler_raw() -> MsgPackRaw<User> {
    let user = User {
        name: "steve".to_string(),
        data: vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
    };
    MsgPack(user)
}

// axum handler | MsgPackRaw
async fn post_handler_raw(MsgPackRaw(user): MsgPackRaw<User>) -> Html<String> {
    let string = format!("<h1>{:?}</h1>", user.name);
    Html(string)
}
```

Dependencies for serializing/deserializing MsgPack
```toml
serde = { version = "1.0", features = ["derive"] }
serde_bytes = "0.11"
```

in order to pack arrays correct remember to add `#[serde(with = "serde_bytes")]` to the struct member.


## Safety
This crate uses #![forbid(unsafe_code)] to ensure everything is implemented in 100% safe Rust.




## License

This project is licensed under the [MIT license][license].


[docs]: https://docs.rs/axum-msgpack
[license]: /axum/LICENSE