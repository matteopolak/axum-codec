use axum::response::{IntoResponse, Response};

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
impl<R> IntoCodecResponse for R
where
	R: IntoResponse,
{
	fn into_codec_response(self, _content_type: ContentType) -> Response {
		self.into_response()
	}
}

#[cfg(feature = "aide")]
impl<R> IntoCodecResponse for R
where
	R: IntoResponse + aide::OperationOutput,
{
	fn into_codec_response(self, _content_type: ContentType) -> Response {
		self.into_response()
	}
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
