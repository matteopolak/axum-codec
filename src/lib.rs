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
pub use extract::*;
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
