use axum::http::HeaderValue;

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
	#[cfg(feature = "cbor")]
	Cbor,
	#[cfg(feature = "yaml")]
	Yaml,
	#[cfg(feature = "toml")]
	Toml,
}

#[cfg(not(any(
	feature = "json",
	feature = "msgpack",
	feature = "bincode",
	feature = "bitcode",
	feature = "cbor",
	feature = "yaml",
	feature = "toml"
)))]
const _: () = {
	compile_error!("At least one of the following features must be enabled: `json`, `msgpack`, `bincode`, `bitcode`, `cbor`, `yaml`, `toml`.");

	impl Default for ContentType {
		fn default() -> Self {
			unreachable!()
		}
	}
};

impl Default for ContentType {
	#[allow(unreachable_code)]
	fn default() -> Self {
		#[cfg(feature = "json")]
		return Self::Json;
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

		Some(match (mime.type_().as_str(), subtype.as_str()) {
			#[cfg(feature = "json")]
			("application", "json") => Self::Json,
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
			_ => return None,
		})
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
			#[cfg(feature = "cbor")]
			Self::Cbor => "application/cbor",
			#[cfg(feature = "yaml")]
			Self::Yaml => "application/x-yaml",
			#[cfg(feature = "toml")]
			Self::Toml => "text/toml",
		};

		HeaderValue::from_static(text)
	}
}
