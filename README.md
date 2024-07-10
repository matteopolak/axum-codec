# Axum Codec

A body extractor for the [Axum](https://github.com/tokio-rs/axum) web framework.

## Features

- Supports encoding and decoding of various formats with a single extractor.
- Provides a wrapper for [`axum::routing::method_routing`](https://docs.rs/axum/latest/axum/routing/method_routing/index.html) to automatically encode responses in the correct format according to the specified `Accept` header (with a fallback to `Content-Type`, then one of the enabled formats).
- Provides an attribute macro (under the `macros` feature) to derive all enabled formats for a struct.

Here's a quick example that can do the following:
- Decode a `User` from the request body in any of the supported formats.
- Encode a `Greeting` to the response body in any of the supported formats.

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

# Feature flags

- `std`*: Enables various standard library features for dependency crates.
- `json`*: Enables [`JSON`](https://github.com/serde-rs/json) support.
- `msgpack`: Enables [`MessagePack`](https://github.com/3Hren/msgpack-rust) support.
- `bincode`: Enables [`Bincode`](https://github.com/bincode-org/bincode) support.
- `bitcode`: Enables [`Bitcode`](https://github.com/SoftbearStudios/bitcode) support.
- `yaml`: Enables [`YAML`](https://github.com/dtolnay/serde-yaml/releases) support.
- `toml`: Enables [`TOML`](https://github.com/toml-rs/toml) support.

* Enabled by default.

## License

Dual-licensed under MIT or Apache License v2.0.

