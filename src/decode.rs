use crate::{codec_trait, Codec, CodecRejection, ContentType};

#[cfg(all(feature = "serde", feature = "bincode", feature = "bitcode"))]
codec_trait!(
	CodecDecode,
	serde::de::DeserializeOwned + bincode::Decode + bitcode::DecodeOwned
);

#[cfg(all(feature = "serde", feature = "bincode", not(feature = "bitcode")))]
codec_trait!(CodecDecode, serde::de::DeserializeOwned + bincode::Decode);

#[cfg(all(feature = "serde", not(feature = "bincode"), feature = "bitcode"))]
codec_trait!(
	CodecDecode,
	serde::de::DeserializeOwned + bitcode::DecodeOwned
);

#[cfg(all(feature = "serde", not(feature = "bincode"), not(feature = "bitcode")))]
codec_trait!(CodecDecode, serde::de::DeserializeOwned);

#[cfg(all(not(feature = "serde"), feature = "bincode", feature = "bitcode"))]
codec_trait!(CodecDecode, bincode::Decode + bitcode::DecodeOwned);

#[cfg(all(not(feature = "serde"), feature = "bincode", not(feature = "bitcode")))]
codec_trait!(CodecDecode, bincode::Decode);

#[cfg(all(not(feature = "serde"), not(feature = "bincode"), feature = "bitcode"))]
codec_trait!(CodecDecode, bitcode::DecodeOwned);

#[cfg(all(
	not(feature = "serde"),
	not(feature = "bincode"),
	not(feature = "bitcode")
))]
codec_trait!(CodecDecode);

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
	///
	/// # Errors
	///
	/// See [`rmp_serde::from_slice`].
	#[cfg(feature = "msgpack")]
	#[inline]
	pub fn from_msgpack(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
		rmp_serde::from_slice(bytes).map(Self)
	}

	/// Attempts to deserialize the given text as [YAML](https://yaml.org).
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
		Ok(match content_type {
			#[cfg(feature = "json")]
			ContentType::Json => Self::from_json(bytes)?,
			#[cfg(feature = "msgpack")]
			ContentType::MsgPack => Self::from_msgpack(bytes)?,
			#[cfg(feature = "bincode")]
			ContentType::Bincode => Self::from_bincode(bytes)?,
			#[cfg(feature = "bitcode")]
			ContentType::Bitcode => Self::from_bitcode(bytes)?,
			#[cfg(feature = "yaml")]
			ContentType::Yaml => Self::from_yaml(core::str::from_utf8(bytes)?)?,
			#[cfg(feature = "toml")]
			ContentType::Toml => Self::from_toml(core::str::from_utf8(bytes)?)?,
		})
	}
}
