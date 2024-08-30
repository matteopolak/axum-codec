#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(
	not(any(
		feature = "json",
		feature = "msgpack",
		feature = "bincode",
		feature = "bitcode",
		feature = "cbor",
		feature = "yaml",
		feature = "toml"
	)),
	allow(unreachable_code, unused_variables)
)]
#![doc = include_str!("../README.md")]

mod content;
mod decode;
mod encode;
pub mod extract;
pub mod handler;
pub mod rejection;
pub mod response;
pub mod routing;

pub use content::{Accept, ContentType};
pub use decode::CodecDecode;
pub use encode::CodecEncode;
pub use extract::Codec;
pub use handler::CodecHandler;
pub use rejection::CodecRejection;
pub use response::IntoCodecResponse;

#[doc(hidden)]
pub mod __private {
	#[cfg(feature = "bincode")]
	pub use bincode;
	#[cfg(feature = "bitcode")]
	pub use bitcode;
	#[cfg(feature = "aide")]
	pub use schemars;
	#[cfg(feature = "serde")]
	pub use serde;
	#[cfg(feature = "validator")]
	pub use validator;
}

pub use axum_codec_macros as macros;
#[cfg(feature = "macros")]
pub use macros::apply;
#[cfg(feature = "macros")]
pub use macros::debug_handler;
#[cfg(feature = "macros")]
pub use macros::debug_middleware;

#[cfg(test)]
mod test {
	use super::*;

	#[apply(decode, encode)]
	#[derive(Debug, PartialEq)]
	struct Data {
		string: String,
		integer: i32,
		array: Vec<i32>,
		boolean: bool,
	}

	fn data() -> Data {
		Data {
			string: "hello".into(),
			integer: 42,
			array: vec![1, 2, 3],
			boolean: true,
		}
	}

	#[test]
	fn test_msgpack_roundtrip() {
		let data = data();
		let encoded = Codec(&data).to_msgpack().unwrap();

		let Codec(decoded) = Codec::<Data>::from_msgpack(&encoded).unwrap();

		assert_eq!(decoded, data);
	}

	#[test]
	fn test_json_roundtrip() {
		let data = data();
		let encoded = Codec(&data).to_json().unwrap();

		let Codec(decoded) = Codec::<Data>::from_json(&encoded).unwrap();

		assert_eq!(decoded, data);
	}

	#[test]
	fn test_cbor_roundtrip() {
		let data = data();
		let encoded = Codec(&data).to_cbor().unwrap();

		let Codec(decoded) = Codec::<Data>::from_cbor(&encoded).unwrap();

		assert_eq!(decoded, data);
	}

	#[test]
	fn test_yaml_roundtrip() {
		let data = data();
		let encoded = Codec(&data).to_yaml().unwrap();

		let Codec(decoded) = Codec::<Data>::from_yaml(&encoded).unwrap();

		assert_eq!(decoded, data);
	}

	#[test]
	fn test_toml_roundtrip() {
		let data = data();
		let encoded = Codec(&data).to_toml().unwrap();

		let Codec(decoded) = Codec::<Data>::from_toml(&encoded).unwrap();

		assert_eq!(decoded, data);
	}

	#[test]
	fn test_bincode_roundtrip() {
		let data = data();
		let encoded = Codec(&data).to_bincode().unwrap();

		let Codec(decoded) = Codec::<Data>::from_bincode(&encoded).unwrap();

		assert_eq!(decoded, data);
	}

	#[test]
	fn test_bitcode_roundtrip() {
		let encoded = Codec(data()).to_bitcode();

		let Codec(decoded) = Codec::<Data>::from_bitcode(&encoded).unwrap();

		assert_eq!(decoded, data());
	}
}
