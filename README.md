# Axum Codec

A body extractor for the [Axum](https://github.com/tokio-rs/axum) web framework.

```rust
use axum_codec::{Accept, Codec};

#[derive(serde::Deserialize)]
struct SomeInput {
  name: String,
  age: u8,
}

#[derive(serde::Serialize)]
struct SomeOutput {
  message: String,
}

/// Deserialize the request body to `SomeInput` from a supported format (e.g. JSON or MessagePack).
async fn handler(accept: Accept, Codec(body): Codec<SomeInput>) -> impl IntoResponse {
  let message = format!("Hello, {}! You are {} years old.", body.name, body.age);

  /// Serializes the output to the appropriate content type,
  /// extracted from the `Accept` (or `Content-Type` if not present or unsupported) header.
  Codec(SomeOutput { message }).to_response(accept)
}

#[tokio::main]
async fn main() {
  let app = Router::new().route("/greet", post(handler));
  let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000)).await.unwrap();

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

