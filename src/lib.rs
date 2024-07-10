#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![doc = include_str!("../README.md")]

mod decode;
mod encode;
mod rejection;

pub use {decode::CodecDecode, encode::CodecEncode, rejection::CodecRejection};

use axum::{
	body::Bytes,
	extract::{FromRequest, FromRequestParts, Request},
	http::{header, request::Parts, HeaderValue},
	response::{IntoResponse, Response},
};

/// Codec extractor / response.
///
/// The serialized data is not specified, unlike [`axum::Json`]. Upon deserialization, the request's
/// `Content-Type` header is used to determine the format of the data.
///
/// The supported formats are:
/// - `JSON`
/// - `MessagePack`
/// - `Bincode`
/// - `Bitcode`
/// - `YAML`
/// - `TOML`
///
/// By default, only JSON is supported. To enable other formats, use the corresponding feature flags.
///
/// Note that [`IntoResponse`] is not implemented for this type, as the headers are not available
/// when serializing the data. Instead, use [`Codec::to_response`] to create a response with the
/// appropriate `Content-Type` header extracted from the request with [`Accept`].
///
/// # Examples
///
/// ```edition2021
/// # use axum_codec::Codec;
/// # use axum::http::HeaderValue;
/// # use serde_json::json;
/// #
/// # fn main() {
/// let bytes = b"{\"hello\": \"world\"}";
/// let content_type = HeaderValue::from_static("application/json");
///
/// let Codec(data) = Codec::from_bytes(bytes, content_type).unwrap();
///
/// assert_eq!(data, json!({ "hello": "world" }));
/// # }
/// ```
pub struct Codec<T>(pub T);

impl<T> Codec<T>
where
	T: CodecEncode,
{
	pub fn to_response<C: Into<ContentType>>(&self, content_type: C) -> Response {
		let content_type = content_type.into();
		let bytes = match self.to_bytes(content_type) {
			Ok(bytes) => bytes,
			Err(rejection) => return rejection.into_response(),
		};

		([(header::CONTENT_TYPE, content_type.into_header())], bytes).into_response()
	}
}

#[axum::async_trait]
impl<T, S> FromRequest<S> for Codec<T>
where
	T: CodecDecode,
	S: Send + Sync + 'static,
{
	type Rejection = CodecRejection;

	async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
		let content_type = req
			.headers()
			.get(header::CONTENT_TYPE)
			.and_then(ContentType::from_header)
			.ok_or(CodecRejection::UnsupportedContentType)?;

		let bytes = Bytes::from_request(req, state).await?;
		let data = Codec::from_bytes(&bytes, content_type)?;

		Ok(data)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
	#[cfg(feature = "json")]
	Json,
	#[cfg(feature = "msgpack")]
	MsgPack,
	#[cfg(feature = "bincode")]
	Bincode,
	#[cfg(feature = "bitcode")]
	Bitcode,
	#[cfg(feature = "yaml")]
	Yaml,
	#[cfg(feature = "toml")]
	Toml,
}

impl ContentType {
	/// Attempts to parse the given [`HeaderValue`] into a [`ContentType`]
	/// by treating it as a MIME type.
	///
	/// Note that, along with official MIME types, this method also recognizes
	/// some unofficial MIME types that are commonly used in practice.
	///
	/// ```edition2021
	/// # use axum_codec::ContentType;
	/// # use axum::http::HeaderValue;
	/// #
	/// # fn main() {
	/// let header = HeaderValue::from_static("application/json");
	/// let content_type = ContentType::from_header(&header).unwrap();
	///
	/// assert_eq!(content_type, ContentType::Json);
	///
	/// let header = HeaderValue::from_static("application/vnd.msgpack");
	/// let content_type = ContentType::from_header(&header).unwrap();
	///
	/// assert_eq!(content_type, ContentType::MsgPack);
	///
	/// let header = HeaderValue::from_static("application/x-msgpack");
	/// let content_type = ContentType::from_header(&header).unwrap();
	///
	/// assert_eq!(content_type, ContentType::MsgPack);
	/// # }
	pub fn from_header(header: &HeaderValue) -> Option<Self> {
		let mime = header.to_str().ok()?.parse::<mime::Mime>().ok()?;
		let subtype = mime.suffix().map_or_else(|| mime.subtype(), |name| name);

		match (mime.type_().as_str(), subtype.as_str()) {
			#[cfg(feature = "json")]
			("application", "json") => Some(Self::Json),
			#[cfg(feature = "msgpack")]
			("application", "msgpack" | "vnd.msgpack" | "x-msgpack" | "x.msgpack") => Some(Self::MsgPack),
			#[cfg(feature = "bincode")]
			("application", "bincode" | "vnd.bincode" | "x-bincode" | "x.bincode") => Some(Self::Bincode),
			#[cfg(feature = "bitcode")]
			("application", "bitcode" | "vnd.bitcode" | "x-bitcode" | "x.bitcode") => Some(Self::Bitcode),
			#[cfg(feature = "yaml")]
			("application" | "text", "yaml" | "yml" | "x-yaml") => Some(Self::Yaml),
			#[cfg(feature = "toml")]
			("application" | "text", "toml" | "x-toml" | "vnd.toml") => Some(Self::Toml),
			_ => None,
		}
	}

	/// Converts the [`ContentType`] into a [`HeaderValue`].
	///
	/// ```edition2021
	/// # use axum_codec::ContentType;
	/// # use axum::http::HeaderValue;
	/// #
	/// # fn main() {
	/// let content_type = ContentType::Json;
	/// let header = content_type.into_header();
	///
	/// assert_eq!(header, HeaderValue::from_static("application/json"));
	///
	/// let content_type = ContentType::MsgPack;
	/// let header = content_type.into_header();
	///
	/// assert_eq!(header, HeaderValue::from_static("application/vnd.msgpack"));
	///
	/// let content_type = ContentType::Yaml;
	/// let header = content_type.into_header();
	///
	/// assert_eq!(header, HeaderValue::from_static("application/x-yaml"));
	///
	/// let content_type = ContentType::Toml;
	/// let header = content_type.into_header();
	///
	/// assert_eq!(header, HeaderValue::from_static("text/toml"));
	/// # }
	#[must_use]
	pub fn into_header(self) -> HeaderValue {
		let text = match self {
			#[cfg(feature = "json")]
			Self::Json => "application/json",
			#[cfg(feature = "msgpack")]
			Self::MsgPack => "application/vnd.msgpack",
			#[cfg(feature = "bincode")]
			Self::Bincode => "application/vnd.bincode",
			#[cfg(feature = "bitcode")]
			Self::Bitcode => "application/vnd.bitcode",
			#[cfg(feature = "yaml")]
			Self::Yaml => "application/x-yaml",
			#[cfg(feature = "toml")]
			Self::Toml => "text/toml",
		};

		HeaderValue::from_static(text)
	}
}

/// Extractor for the request's desired response [`ContentType`].
///
/// # Examples
///
/// ```edition2021
/// # use axum_codec::{Accept, Codec};
/// # use axum::{http::HeaderValue, response::IntoResponse};
/// # use serde::Serialize;
/// #
/// # fn main() {
/// #[derive(Serialize)]
/// struct User {
///   name: String,
///   age: u8,
/// }
///
/// fn get_user(accept: Accept) -> impl IntoResponse {
///   Codec(User {
///     name: "Alice".into(),
///     age: 42,
///   })
///   .to_response(accept)
/// }
/// #
/// # fn main() {}
#[derive(Debug, Clone, Copy)]
pub struct Accept(ContentType);

impl Accept {
	/// Returns the request's desired response [`ContentType`].
	#[inline]
	#[must_use]
	pub fn content_type(self) -> ContentType {
		self.0
	}
}

impl From<Accept> for ContentType {
	#[inline]
	fn from(accept: Accept) -> Self {
		accept.0
	}
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for Accept
where
	S: Send + Sync + 'static,
{
	type Rejection = CodecRejection;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		let header = None
			.or_else(|| parts.headers.get(header::ACCEPT))
			.or_else(|| parts.headers.get(header::CONTENT_TYPE))
			.and_then(ContentType::from_header)
			.ok_or(CodecRejection::UnsupportedContentType)?;

		Ok(Self(header))
	}
}

/// Defines the [`CodecDecode`] and [`CodecEncode`] traits with the given constraints.
macro_rules! codec_trait {
	($id:ident, $($constraint:tt)*) => {
		pub trait $id: $($constraint)* {}
		impl<T> $id for T where T: $($constraint)* {}
	};
	($id:ident) => {
		pub trait $id {}
		impl<T> $id for T {}
	};
}

pub(crate) use codec_trait;
