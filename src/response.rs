use axum::response::Response;

use crate::{Codec, CodecEncode, ContentType};

#[cfg(not(feature = "aide"))]
pub trait IntoCodecResponse {
	fn into_codec_response(self, content_type: ContentType) -> Response;
}

#[cfg(feature = "aide")]
pub trait IntoCodecResponse: aide::OperationOutput {
	fn into_codec_response(self, content_type: ContentType) -> Response;
}

#[cfg(not(feature = "aide"))]
impl<D> IntoCodecResponse for Codec<D>
where
	D: CodecEncode,
{
	fn into_codec_response(self, content_type: ContentType) -> Response {
		self.to_response(content_type)
	}
}

#[cfg(feature = "aide")]
impl<D> IntoCodecResponse for Codec<D>
where
	D: CodecEncode,
	Self: aide::OperationOutput,
{
	fn into_codec_response(self, content_type: ContentType) -> Response {
		self.to_response(content_type)
	}
}

mod axum_impls {
	use std::borrow::Cow;

	use super::{ContentType, IntoCodecResponse};

	use axum::{
		body::Bytes,
		http::StatusCode,
		response::{IntoResponse, Response},
		BoxError,
	};

	impl<T, E> IntoCodecResponse for Result<T, E>
	where
		T: IntoCodecResponse,
		E: IntoCodecResponse,
	{
		fn into_codec_response(self, content_type: ContentType) -> Response {
			match self {
				Ok(value) => value.into_codec_response(content_type),
				Err(err) => err.into_codec_response(content_type),
			}
		}
	}

	impl<B> IntoCodecResponse for Response<B>
	where
		B: axum::body::HttpBody<Data = Bytes> + Send + 'static,
		B::Error: Into<BoxError>,
	{
		fn into_codec_response(self, _ct: ContentType) -> Response {
			self.into_response()
		}
	}

	macro_rules! forward_to_into_response {
		( $($ty:ty),* ) => {
				$(
						impl IntoCodecResponse for $ty {
								fn into_codec_response(self, _ct: ContentType) -> Response {
										self.into_response()
								}
						}
				)*
		}
	}

	forward_to_into_response! {
		StatusCode, (), &'static str, String, Bytes, Cow<'static, str>, &'static [u8], Vec<u8>,  Cow<'static, [u8]>
	}

	impl<R> IntoCodecResponse for (StatusCode, R)
	where
		R: IntoCodecResponse,
	{
		fn into_codec_response(self, content_type: ContentType) -> Response {
			let mut res = self.1.into_codec_response(content_type);
			*res.status_mut() = self.0;
			res
		}
	}
}
