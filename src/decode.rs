use crate::{Codec, CodecRejection, ContentType};

crate::macros::__private_decode_trait! {
	/// Decoder trait for deserializing bytes into all supported formats.
	///
	/// Note that feature flags affect this trait differently than normal. In this case,
	/// feature flags further restrict the trait instead of being additive. This may change
	/// in the future.
}

#[cfg(feature = "serde")]
impl<T> Codec<T>
where
	T: serde::de::DeserializeOwned,
{
	/// Attempts to deserialize the given bytes as [JSON](https://www.json.org).
	///
	/// # Errors
	///
	/// See [`serde_json::from_slice`].
	#[cfg(feature = "json")]
	#[inline]
	pub fn from_json(bytes: &[u8]) -> Result<Self, serde_json::Error> {
		serde_json::from_slice(bytes).map(Self)
	}

	/// Attempts to deserialize the given bytes as [MessagePack](https://msgpack.org).
	/// Does not perform any validation if the `validator` feature is enabled. For validation,
	/// use [`Self::from_bytes`].
	///
	/// # Errors
	///
	/// See [`rmp_serde::from_slice`].
	#[cfg(feature = "msgpack")]
	#[inline]
	pub fn from_msgpack(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
		rmp_serde::from_slice(bytes).map(Self)
	}

	/// Attemps to deserialize the given bytes as [CBOR](https://cbor.io).
	/// Does not perform any validation if the `validator` feature is enabled. For validation,
	/// use [`Self::from_bytes`].
	///
	/// # Errors
	///
	/// See [`ciborium::from_slice`].
	#[cfg(feature = "cbor")]
	#[inline]
	pub fn from_cbor(bytes: &[u8]) -> Result<Self, ciborium::de::Error<std::io::Error>> {
		ciborium::from_reader(bytes).map(Self)
	}

	/// Attempts to deserialize the given text as [YAML](https://yaml.org).
	/// Does not perform any validation if the `validator` feature is enabled. For validation,
	/// use [`Self::from_bytes`].
	///
	/// # Errors
	///
	/// See [`serde_yaml::from_slice`].
	#[cfg(feature = "yaml")]
	#[inline]
	pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
		serde_yaml::from_str(text).map(Self)
	}

	/// Attempts to deserialize the given text as [TOML](https://toml.io).
	/// Does not perform any validation if the `validator` feature is enabled. For validation,
	/// use [`Self::from_bytes`].
	///
	/// # Errors
	///
	/// See [`toml::from_str`].
	#[cfg(feature = "toml")]
	#[inline]
	pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
		toml::from_str(text).map(Self)
	}
}

impl<T> Codec<T> {
	/// Attempts to deserialize the given bytes as [Bincode](https://github.com/bincode-org/bincode).
	/// Does not perform any validation if the `validator` feature is enabled. For validation,
	/// use [`Self::from_bytes`].
	///
	/// # Errors
	///
	/// See [`bincode::decode_from_slice`].
	#[cfg(feature = "bincode")]
	#[inline]
	pub fn from_bincode(bytes: &[u8]) -> Result<Self, bincode::error::DecodeError>
	where
		T: bincode::Decode,
	{
		bincode::decode_from_slice(bytes, bincode::config::standard()).map(|t| Self(t.0))
	}

	/// Attempts to deserialize the given bytes as [Bitcode](https://github.com/SoftbearStudios/bitcode).
	/// Does not perform any validation if the `validator` feature is enabled. For validation,
	/// use [`Self::from_bytes`].
	///
	/// # Errors
	///
	/// See [`bitcode::decode`].
	#[cfg(feature = "bitcode")]
	#[inline]
	pub fn from_bitcode(bytes: &[u8]) -> Result<Self, bitcode::Error>
	where
		T: bitcode::DecodeOwned,
	{
		bitcode::decode(bytes).map(Self)
	}

	/// Attempts to deserialize the given bytes as the specified [`ContentType`].
	///
	/// # Errors
	///
	/// See [`CodecRejection`].
	pub fn from_bytes(bytes: &[u8], content_type: ContentType) -> Result<Self, CodecRejection>
	where
		T: CodecDecode,
	{
		let codec = match content_type {
			#[cfg(feature = "json")]
			ContentType::Json => Self::from_json(bytes)?,
			#[cfg(feature = "msgpack")]
			ContentType::MsgPack => Self::from_msgpack(bytes)?,
			#[cfg(feature = "bincode")]
			ContentType::Bincode => Self::from_bincode(bytes)?,
			#[cfg(feature = "bitcode")]
			ContentType::Bitcode => Self::from_bitcode(bytes)?,
			#[cfg(feature = "cbor")]
			ContentType::Cbor => Self::from_cbor(bytes)?,
			#[cfg(feature = "yaml")]
			ContentType::Yaml => Self::from_yaml(core::str::from_utf8(bytes)?)?,
			#[cfg(feature = "toml")]
			ContentType::Toml => Self::from_toml(core::str::from_utf8(bytes)?)?,
		};

		#[cfg(feature = "validator")]
		validator::Validate::validate(&codec)?;

		Ok(codec)
	}
}
