use core::fmt;
use std::{convert::Infallible, str::FromStr};

use axum::{
	extract::FromRequestParts,
	http::{header, request::Parts, HeaderValue},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentType {
	#[cfg(feature = "json")]
	Json,
	#[cfg(feature = "form")]
	Form,
	#[cfg(feature = "msgpack")]
	MsgPack,
	#[cfg(feature = "bincode")]
	Bincode,
	#[cfg(feature = "bitcode")]
	Bitcode,
	#[cfg(feature = "cbor")]
	Cbor,
	#[cfg(feature = "yaml")]
	Yaml,
	#[cfg(feature = "toml")]
	Toml,
}

#[cfg(not(any(
	feature = "json",
	feature = "form",
	feature = "msgpack",
	feature = "bincode",
	feature = "bitcode",
	feature = "cbor",
	feature = "yaml",
	feature = "toml"
)))]
const _: () = {
	compile_error!(
		"At least one of the following features must be enabled: `json`, `form`, `msgpack`, \
		 `bincode`, `bitcode`, `cbor`, `yaml`, `toml`."
	);

	impl Default for ContentType {
		fn default() -> Self {
			unimplemented!()
		}
	}
};

#[cfg(any(
	feature = "json",
	feature = "form",
	feature = "msgpack",
	feature = "bincode",
	feature = "bitcode",
	feature = "cbor",
	feature = "yaml",
	feature = "toml"
))]
impl Default for ContentType {
	#[allow(unreachable_code)]
	fn default() -> Self {
		#[cfg(feature = "json")]
		return Self::Json;
		#[cfg(feature = "form")]
		return Self::Form;
		#[cfg(feature = "msgpack")]
		return Self::MsgPack;
		#[cfg(feature = "bincode")]
		return Self::Bincode;
		#[cfg(feature = "bitcode")]
		return Self::Bitcode;
		#[cfg(feature = "cbor")]
		return Self::Cbor;
		#[cfg(feature = "yaml")]
		return Self::Yaml;
		#[cfg(feature = "toml")]
		return Self::Toml;
	}
}

impl fmt::Display for ContentType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

#[derive(Debug, thiserror::Error)]
pub enum FromStrError {
	#[error("invalid content type")]
	InvalidContentType,
	#[error(transparent)]
	Mime(#[from] mime::FromStrError),
}

impl FromStr for ContentType {
	type Err = FromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {

		let accept: accept_header::Accept = s.parse().map_err(|_| FromStrError::InvalidContentType)?;

		// Prepare a list of supported media types, based on which features are enabled
		let mut available = Vec::new();
		#[cfg(feature = "json")]
		available.push("application/json");
		#[cfg(feature = "form")]
		available.push("application/x-www-form-urlencoded");
		#[cfg(feature = "msgpack")]
		available.extend([ "application/msgpack", "application/vnd.msgpack", "application/x-msgpack", "application/x.msgpack"]);
		#[cfg(feature = "bincode")]
		available.extend([ "application/bincode", "application/vnd.bincode", "application/x-bincode", "application/x.bincode"]);
		#[cfg(feature = "bitcode")]
		available.extend([ "application/bitcode", "application/vnd.bitcode", "application/x-bitcode", "application/x.bitcode"]);
		#[cfg(feature = "cbor")]
		available.push("application/cbor");
		#[cfg(feature = "yaml")]
		available.extend([ "application/yaml", "application/yml", "application/x-yaml", "text/yaml", "text/yml", "text/x-yaml"]);
		#[cfg(feature = "toml")]
		available.extend([ "application/toml", "application/x-toml", "application/vnd.toml", "text/toml", "text/x-toml", "text/vnd.toml"]);

		let available_mimes: Vec<mime::Mime> = available.into_iter().map(|string| mime::Mime::from_str(string).unwrap()).collect();
		let mime = accept.negotiate(&available_mimes).map_err(|_| FromStrError::InvalidContentType)?;
		let subtype = mime.suffix().unwrap_or_else(|| mime.subtype());

		Ok(match (mime.type_().as_str(), subtype.as_str()) {
			#[cfg(feature = "json")]
			("application", "json") => Self::Json,
			#[cfg(feature = "form")]
			("application", "x-www-form-urlencoded") => Self::Form,
			#[cfg(feature = "msgpack")]
			("application", "msgpack" | "vnd.msgpack" | "x-msgpack" | "x.msgpack") => Self::MsgPack,
			#[cfg(feature = "bincode")]
			("application", "bincode" | "vnd.bincode" | "x-bincode" | "x.bincode") => Self::Bincode,
			#[cfg(feature = "bitcode")]
			("application", "bitcode" | "vnd.bitcode" | "x-bitcode" | "x.bitcode") => Self::Bitcode,
			#[cfg(feature = "cbor")]
			("application", "cbor") => Self::Cbor,
			#[cfg(feature = "yaml")]
			("application" | "text", "yaml" | "yml" | "x-yaml") => Self::Yaml,
			#[cfg(feature = "toml")]
			("application" | "text", "toml" | "x-toml" | "vnd.toml") => Self::Toml,
			_ => return Err(FromStrError::InvalidContentType),
		})
	}
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
	///
	/// let header = HeaderValue::from_static("text/plain, image/png, application/x-yaml, application/json");
	/// let content_type = ContentType::from_header(&header).unwrap();
	///
	/// assert_eq!(content_type, ContentType::Yaml);
	///
	/// let header = HeaderValue::from_static("application/x-msgpack;q=0.7, application/json;q=0.6");
	/// let content_type = ContentType::from_header(&header).unwrap();
	///
	/// assert_eq!(content_type, ContentType::MsgPack);
	///
	/// let header = HeaderValue::from_static("text/plain, image/png, unknown/*");
	/// let option = ContentType::from_header(&header);
	///
	/// assert_eq!(option, None);
	///
	/// let header = HeaderValue::from_static("");
	/// let option = ContentType::from_header(&header);
	///
	/// assert_eq!(option, None);
	/// # }
	pub fn from_header(header: &HeaderValue) -> Option<Self> {
		header.to_str().ok()?.parse().ok()
	}

	/// Returns the MIME type as a string slice.
	///
	/// ```edition2021
	/// # use axum_codec::ContentType;
	/// #
	/// let content_type = ContentType::Json;
	///
	/// assert_eq!(content_type.as_str(), "application/json");
	/// ```
	#[must_use]
	pub fn as_str(&self) -> &'static str {
		match *self {
			#[cfg(feature = "json")]
			Self::Json => "application/json",
			#[cfg(feature = "form")]
			Self::Form => "application/x-www-form-urlencoded",
			#[cfg(feature = "msgpack")]
			Self::MsgPack => "application/vnd.msgpack",
			#[cfg(feature = "bincode")]
			Self::Bincode => "application/vnd.bincode",
			#[cfg(feature = "bitcode")]
			Self::Bitcode => "application/vnd.bitcode",
			#[cfg(feature = "cbor")]
			Self::Cbor => "application/cbor",
			#[cfg(feature = "yaml")]
			Self::Yaml => "application/x-yaml",
			#[cfg(feature = "toml")]
			Self::Toml => "text/toml",
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
		HeaderValue::from_static(self.as_str())
	}
}

impl<S> FromRequestParts<S> for ContentType
where
	S: Sync,
{
	type Rejection = Infallible;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		let header = parts
			.headers
			.get(header::CONTENT_TYPE)
			.and_then(Self::from_header);

		Ok(header.unwrap_or_default())
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
/// #[axum_codec::apply(encode)]
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
/// ```
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

impl<S> FromRequestParts<S> for Accept
where
	S: Send + Sync + 'static,
{
	type Rejection = Infallible;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		let header = None
			.or_else(|| {
				parts
					.headers
					.get(header::ACCEPT)
					.and_then(ContentType::from_header)
			})
			.or_else(|| {
				parts
					.headers
					.get(header::CONTENT_TYPE)
					.and_then(ContentType::from_header)
			})
			.unwrap_or_default();

		Ok(Self(header))
	}
}
