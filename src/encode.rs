use axum::response::{IntoResponse, Response};

use crate::{Codec, ContentType};

crate::macros::__private_encode_trait! {
	/// Encoder trait for encoding a value into any supported format.
	///
	/// Note that feature flags affect this trait differently than normal. In this case,
	/// feature flags further restrict the trait instead of being additive. This may change
	/// in the future.
}

/// Errors that can occur during encoding.
///
/// In debug mode this will include the error message. In release mode it will
/// only include a status code of `500 Internal Server Error`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[cfg(feature = "json")]
	#[error(transparent)]
	Json(#[from] serde_json::Error),
	#[cfg(feature = "msgpack")]
	#[error(transparent)]
	MsgPack(#[from] rmp_serde::encode::Error),
	#[cfg(feature = "cbor")]
	#[error(transparent)]
	Cbor(#[from] ciborium::ser::Error<std::io::Error>),
	#[cfg(feature = "bincode")]
	#[error(transparent)]
	Bincode(#[from] bincode::error::EncodeError),
	#[cfg(feature = "yaml")]
	#[error(transparent)]
	Yaml(#[from] serde_yaml::Error),
	#[cfg(feature = "toml")]
	#[error(transparent)]
	Toml(#[from] toml::ser::Error),
}

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		use axum::http::StatusCode;

		#[cfg(debug_assertions)]
		return (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response();
		#[cfg(not(debug_assertions))]
		StatusCode::INTERNAL_SERVER_ERROR.into_response()
	}
}

#[cfg(feature = "serde")]
impl<T> Codec<T>
where
	T: serde::Serialize,
{
	/// Attempts to serialize the given value as [JSON](https://www.json.org).
	///
	/// # Errors
	///
	/// See [`serde_json::to_vec`].
	#[cfg(feature = "json")]
	#[inline]
	pub fn to_json(&self) -> Result<Vec<u8>, serde_json::Error> {
		serde_json::to_vec(&self.0)
	}

	/// Attempts to serialize the given value as [MessagePack](https://msgpack.org).
	///
	/// # Errors
	///
	/// See [`rmp_serde::to_vec_named`].
	#[cfg(feature = "msgpack")]
	#[inline]
	pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
		rmp_serde::to_vec_named(&self.0)
	}

	/// Attempts to serialize the given value as [CBOR](https://cbor.io).
	///
	/// # Errors
	///
	/// See [`ciborium::into_writer`].
	#[cfg(feature = "cbor")]
	#[inline]
	pub fn to_cbor(&self) -> Result<Vec<u8>, ciborium::ser::Error<std::io::Error>> {
		let mut buf = Vec::new();
		ciborium::into_writer(&self.0, &mut buf)?;
		Ok(buf)
	}

	/// Attempts to serialize the given value as [YAML](https://yaml.org).
	///
	/// # Errors
	///
	/// See [`serde_yaml::to_vec`].
	#[cfg(feature = "yaml")]
	#[inline]
	pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
		serde_yaml::to_string(&self.0)
	}

	/// Attempts to serialize the given value as [TOML](https://toml.io).
	///
	/// # Errors
	///
	/// See [`toml::to_string`].
	#[cfg(feature = "toml")]
	#[inline]
	pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
		toml::to_string(&self.0)
	}
}

impl<T> Codec<T> {
	/// Attempts to serialize the given value as [Bincode]()
	///
	/// # Errors
	///
	/// See [`bincode::serialize`].
	#[cfg(feature = "bincode")]
	#[inline]
	pub fn to_bincode(&self) -> Result<Vec<u8>, bincode::error::EncodeError>
	where
		T: bincode::Encode,
	{
		bincode::encode_to_vec(&self.0, bincode::config::standard())
	}

	/// Attempts to serialize the given value as [Bitcode]()
	///
	/// # Errors
	///
	/// See [`bitcode::encode`].
	#[cfg(feature = "bitcode")]
	#[inline]
	pub fn to_bitcode(&self) -> Vec<u8>
	where
		T: bitcode::Encode,
	{
		bitcode::encode(&self.0)
	}

	/// Attempts to serialize the given value as the specified [`ContentType`].
	///
	/// # Errors
	///
	/// See [`CodecRejection`].
	pub fn to_bytes(&self, content_type: ContentType) -> Result<Vec<u8>, Error>
	where
		T: CodecEncode,
	{
		Ok(match content_type {
			#[cfg(feature = "json")]
			ContentType::Json => self.to_json()?,
			#[cfg(feature = "msgpack")]
			ContentType::MsgPack => self.to_msgpack()?,
			#[cfg(feature = "bincode")]
			ContentType::Bincode => self.to_bincode()?,
			#[cfg(feature = "bitcode")]
			ContentType::Bitcode => self.to_bitcode(),
			#[cfg(feature = "cbor")]
			ContentType::Cbor => self.to_cbor()?,
			#[cfg(feature = "yaml")]
			ContentType::Yaml => self.to_yaml()?.into_bytes(),
			#[cfg(feature = "toml")]
			ContentType::Toml => self.to_toml()?.into_bytes(),
		})
	}
}
