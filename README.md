# Axum Codec

[![https://img.shields.io/crates/v/axum-codec](https://img.shields.io/crates/v/axum-codec)](https://crates.io/crates/axum-codec)
[![https://img.shields.io/docsrs/axum-codec](https://img.shields.io/docsrs/axum-codec)](https://docs.rs/axum-codec/latest/axum_codec/)
[![ci status](https://github.com/matteopolak/axum-codec/workflows/ci/badge.svg)](https://github.com/matteopolak/axum-codec/actions)

A body extractor for the [Axum](https://github.com/tokio-rs/axum) web framework.

## Features

- Supports encoding and decoding of various formats with a single extractor.
- Provides a wrapper for [`axum::routing::method_routing`](https://docs.rs/axum/latest/axum/routing/method_routing/index.html) to automatically encode responses in the correct format according to the specified `Accept` header (with a fallback to `Content-Type`, then one of the enabled formats).
- Provides an attribute macro (under the `macros` feature) to add derives for all enabled formats to a struct/enum.

## Todo

- [x] Support `bitcode`, `bincode`, `rmp`, `toml`, `serde_yaml`, and `serde_json`
- [x] Add custom `MethodRouter` to automatically encode responses in the correct format
- [x] Add macro to derive all enabled formats for a struct/enum
- [x] Add support for [`aide`](https://github.com/tamasfe/aide)
- [x] Add support for [`validator`](https://github.com/Keats/validator)
- [ ] Support more formats (issues and PRs welcome)
- [ ] Add benchmarks?

Here's a quick example that can do the following:
- Decode a `User` from the request body in any of the supported formats.
- Encode a `Greeting` to the response body in any of the supported formats.

```rust
use axum::{Router, response::IntoResponse};
use axum_codec::{
  handler::IntoCodecResponse,
  routing::{get, post},
  Codec,
  extract::Accept,
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
// Same goes for `validator`.
#[axum_codec::apply(encode, decode)]
struct User {
  name: String,
  age: u8,
}

async fn me() -> User {
  User {
    name: "Alice".into(),
    age: 42,
  }
}

/// A manual implementation of the handler above.
async fn manual_me(accept: Accept, Codec(user): Codec<User>) -> impl IntoResponse {
  Codec(User {
    name: "Alice".into(),
    age: 42,
  })
  .into_codec_response(accept.content_type())
}

#[axum_codec::apply(encode)]
struct Greeting {
  message: String,
}

/// Specify `impl IntoCodecResponse`, similar to `axum`
async fn greet(Codec(user): Codec<User>) -> impl IntoCodecResponse {
  Greeting {
    message: format!("Hello, {}! You are {} years old.", user.name, user.age),
  }
}

#[tokio::main]
async fn main() {
  let app: Router = Router::new()
    .route("/me", get(me).into())
    .route("/manual", axum::routing::post(manual_me))
    .route("/greet", post(greet).into());

  let listener = tokio::net::TcpListener::bind(("127.0.0.1", 3000))
    .await
    .unwrap();

  // axum::serve(listener, app).await.unwrap();
}
```

# Feature flags

- `macros`: Enables the `axum_codec::apply` attribute macro.
- `json`*: Enables [`JSON`](https://github.com/serde-rs/json) support.
- `msgpack`: Enables [`MessagePack`](https://github.com/3Hren/msgpack-rust) support.
- `bincode`: Enables [`Bincode`](https://github.com/bincode-org/bincode) support.
- `bitcode`: Enables [`Bitcode`](https://github.com/SoftbearStudios/bitcode) support.
- `yaml`: Enables [`YAML`](https://github.com/dtolnay/serde-yaml/releases) support.
- `toml`: Enables [`TOML`](https://github.com/toml-rs/toml) support.
- `aide`: Enables support for the [`Aide`](https://github.com/tamasfe/aide) documentation library.
- `validator`: Enables support for the [`Validator`](https://github.com/Keats/validator) validation library, validating all input when extracted with `Codec<T>`.

* Enabled by default.

## License

Dual-licensed under MIT or Apache License v2.0.

