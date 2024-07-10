use axum::{
	extract::rejection::BytesRejection,
	response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum CodecRejection {
	#[error("unsupported content type")]
	UnsupportedContentType,
	#[error(transparent)]
	Bytes(#[from] BytesRejection),
	#[cfg(feature = "json")]
	#[error(transparent)]
	Json(#[from] serde_json::Error),
	#[cfg(feature = "msgpack")]
	#[error(transparent)]
	MsgPack(#[from] rmp_serde::decode::Error),
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
	#[cfg(feature = "toml")]
	#[error(transparent)]
	Utf8Error(#[from] core::str::Utf8Error),
}

impl IntoResponse for CodecRejection {
	fn into_response(self) -> Response {
		use axum::http::StatusCode;

		let status = match &self {
			Self::UnsupportedContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
			_ => StatusCode::BAD_REQUEST,
		};

		(status, self.to_string()).into_response()
	}
}
