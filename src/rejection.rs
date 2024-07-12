use axum::{extract::rejection::BytesRejection, response::Response};

use crate::{handler::IntoCodecResponse, ContentType};

/// Rejection used for [`Codec`](crate::Codec).
///
/// Contains one variant for each way the [`Codec`](crate::Codec) extractor
/// can fail.
#[derive(Debug, thiserror::Error)]
pub enum CodecRejection {
	#[error(transparent)]
	Bytes(#[from] BytesRejection),
	#[cfg(feature = "json")]
	#[error(transparent)]
	Json(#[from] serde_json::Error),
	#[cfg(feature = "msgpack")]
	#[error(transparent)]
	MsgPack(#[from] rmp_serde::decode::Error),
	#[cfg(feature = "cbor")]
	#[error(transparent)]
	Cbor(#[from] ciborium::de::Error<std::io::Error>),
	#[cfg(feature = "bincode")]
	#[error(transparent)]
	Bincode(#[from] bincode::error::DecodeError),
	#[cfg(feature = "bitcode")]
	#[error(transparent)]
	Bitcode(#[from] bitcode::Error),
	#[cfg(feature = "yaml")]
	#[error(transparent)]
	Yaml(#[from] serde_yaml::Error),
	#[cfg(feature = "toml")]
	#[error(transparent)]
	Toml(#[from] toml::de::Error),
	#[cfg(any(feature = "toml", feature = "yaml"))]
	#[error(transparent)]
	Utf8Error(#[from] core::str::Utf8Error),
	#[cfg(feature = "validator")]
	#[error("validator error")]
	Validator(#[from] validator::ValidationErrors),
}

#[cfg(not(feature = "pretty-errors"))]
impl IntoCodecResponse for CodecRejection {
	fn into_codec_response(self, _content_type: ContentType) -> Response {
		self.to_string().into_response()
	}
}

#[cfg(feature = "aide")]
impl schemars::JsonSchema for CodecRejection {
	fn schema_name() -> String {
		Message::schema_name()
	}

	fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
		Message::json_schema(gen)
	}
}

#[cfg(feature = "pretty-errors")]
impl IntoCodecResponse for CodecRejection {
	fn into_codec_response(self, content_type: ContentType) -> Response {
		crate::Codec(self.message()).into_codec_response(content_type)
	}
}

#[cfg(feature = "pretty-errors")]
use crate as axum_codec;

#[cfg(feature = "pretty-errors")]
#[crate::apply(encode)]
pub struct Message {
	/// A unique error code, useful for localization.
	pub code: &'static str,
	/// A human-readable error message in English.
	// TODO: use Cow<'static, str> when bitcode supports it
	pub content: String,
}

impl CodecRejection {
	/// Consumes the rejection and returns a pretty [`Message`] representing the error.
	///
	/// Useful for sending a detailed error message to the client, but not so much
	/// for local debugging.
	#[cfg(feature = "pretty-errors")]
	#[must_use]
	pub fn message(&self) -> Message {
		let code = match self {
			Self::Bytes(..) => {
				return Message {
					code: "payload_too_large",
					content: "The request payload is too large.".into(),
				}
			}
			#[cfg(feature = "json")]
			Self::Json(..) => "decode",
			#[cfg(feature = "msgpack")]
			Self::MsgPack(..) => "decode",
			#[cfg(feature = "cbor")]
			Self::Cbor(..) => "decode",
			#[cfg(feature = "bincode")]
			Self::Bincode(..) => "decode",
			#[cfg(feature = "bitcode")]
			Self::Bitcode(..) => "decode",
			#[cfg(feature = "yaml")]
			Self::Yaml(..) => "decode",
			#[cfg(feature = "toml")]
			Self::Toml(..) => "decode",
			#[cfg(any(feature = "toml", feature = "yaml"))]
			Self::Utf8Error(..) => {
				return Message {
					code: "malformed_utf8",
					content: "The request payload is not valid UTF-8 when it should be.".into(),
				}
			}
			#[cfg(feature = "validator")]
			Self::Validator(err) => {
				return Message {
					code: "invalid_input",
					content: format_validator(err),
				}
			}
		};

		Message {
			code,
			content: self.to_string(),
		}
	}
}

#[cfg(all(feature = "pretty-errors", feature = "validator"))]
fn format_validator(err: &validator::ValidationErrors) -> String {
	let mut buf = String::new();

	for (field, error) in err.errors() {
		append_validator_errors(field, error, &mut buf);
	}

	buf
}

#[cfg(all(feature = "pretty-errors", feature = "validator"))]
fn append_validator_errors(field: &str, err: &validator::ValidationErrorsKind, buf: &mut String) {
	match err {
		validator::ValidationErrorsKind::Field(errors) => {
			for error in errors {
				if !buf.is_empty() {
					buf.push_str(", ");
				}

				buf.push_str(field);
				buf.push_str(": ");

				if let Some(message) = &error.message {
					buf.push_str(message);
				} else {
					buf.push_str(&error.code);
				}

				if !error.params.is_empty() {
					buf.push_str(" (");

					let mut params = error.params.iter();

					if let Some((key, value)) = params.next() {
						buf.push_str(key);
						buf.push_str(": ");
						buf.push_str(&value.to_string());
					}

					for (key, value) in params {
						buf.push_str(", ");
						buf.push_str(key);
						buf.push_str(": ");
						buf.push_str(&value.to_string());
					}

					buf.push(')');
				}
			}
		}
		validator::ValidationErrorsKind::List(message) => {
			for error in message.values() {
				for (field, errors) in error.errors() {
					append_validator_errors(field, errors, buf);
				}
			}
		}
		validator::ValidationErrorsKind::Struct(struct_) => {
			for (field, errors) in struct_.errors() {
				append_validator_errors(field, errors, buf);
			}
		}
	}
}
