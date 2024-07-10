# Axum Codec

A body extractor for the [Axum](https://github.com/tokio-rs/axum) web framework.

```rust
use axum_codec::{
  handler::IntoCodec,
  routing::{get, post},
  Codec,
};

// Shorthand for the following (assuming all features are enabled):
//
// #[derive(
//   serde::Serialize, serde::Deserialize,
//   bincode::Encode, bincode::Decode,
//   bitcode::Encode, bitcode::Decode,
// )]
// #[serde(crate = "...")]
// #[bincode(crate = "...")]
//
// NOTE: `bitcode` does not support `#[bitcode(crate = "...)]` yet,
// so the dependency must be specified in your `Cargo.toml` if enabled (and using this macro).
#[axum_codec::derive(encode, decode)]
struct User {
  name: String,
  age: u8,
}

async fn me() -> impl IntoCodec<User> {
  User {
    name: "Alice".into(),
    age: 42,
  }
}

#[axum_codec::derive(encode)]
struct Greeting {
  message: String,
}

async fn greet(Codec(user): Codec<User>) -> Greeting {
  Greeting {
    message: format!("Hello, {}! You are {} years old.", user.name, user.age),
  }
}

#[tokio::main]
async fn main() {
  let app = Router::new()
    .route("/me", get(me).into())
    .route("/greet", post(greet).into());

  let listener = tokio::net::TcpListener::bind(("127.0.0.1", 3000))
    .await
    .unwrap();

  axum::serve(listener, app).await.unwrap();
}
```

# Features

- `std`*: Enables various standard library features for dependency crates.
- `json`*: Enables `JSON` support.
- `msgpack`: Enables `MessagePack` support.
- `bincode`: Enables `Bincode` support.
- `bitcode`: Enables `Bitcode` support.
- `yaml`: Enables `YAML` support.
- `toml`: Enables `TOML` support.

* Enabled by default.

## License

Dual-licensed under MIT or Apache License v2.0.

