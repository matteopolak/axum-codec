use core::{
	fmt,
	ops::{Deref, DerefMut},
};

use axum::{
	extract::{FromRequest, FromRequestParts, Request},
	http::header,
	response::{IntoResponse, Response},
};
use bytes::BytesMut;

use crate::{Accept, CodecDecode, CodecEncode, CodecRejection, ContentType, IntoCodecResponse};

/// Codec extractor / response.
///
/// The serialized data is not specified. Upon deserialization, the request's
/// `Content-Type` header is used to determine the format of the data.
///
/// By default, only JSON is supported. To enable other formats, use the
/// corresponding feature flags.
///
/// Note that [`IntoResponse`] is not implemented for this type, as the headers
/// are not available when serializing the data. Instead, use
/// [`Codec::to_response`] to create a response with the appropriate
/// `Content-Type` header extracted from the request with [`Accept`].
///
/// # Examples
///
/// ```edition2021
/// # use axum_codec::{Codec, ContentType};
/// # use axum::http::HeaderValue;
/// # use serde_json::json;
/// #
/// # fn main() {
/// #[axum_codec::apply(decode)]
/// struct Greeting {
///   hello: String,
/// }
///
/// let bytes = b"{\"hello\": \"world\"}";
/// let content_type = ContentType::Json;
///
/// let Codec(data) = Codec::<Greeting>::from_bytes(bytes, content_type).unwrap();
///
/// assert_eq!(data.hello, "world");
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Codec<T>(pub T);

impl<T> Codec<T> {
	/// Consumes the [`Codec`] and returns the inner value.
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T> Codec<T>
where
	T: CodecEncode,
{
	/// Converts the inner value into a response with the given content type.
	///
	/// If serialization fails, the rejection is converted into a response. See
	/// [`encode::Error`](crate::encode::Error) for possible errors.
	pub fn to_response<C: Into<ContentType>>(&self, content_type: C) -> Response {
		let content_type = content_type.into();
		let bytes = match self.to_bytes(content_type) {
			Ok(bytes) => bytes,
			Err(rejection) => return rejection.into_response(),
		};

		([(header::CONTENT_TYPE, content_type.into_header())], bytes).into_response()
	}
}

impl<T> Deref for Codec<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for Codec<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T: fmt::Display> fmt::Display for Codec<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

#[axum::async_trait]
impl<T, S> FromRequest<S> for Codec<T>
where
	T: for<'de> CodecDecode<'de>,
	S: Send + Sync + 'static,
{
	type Rejection = Response;

	async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
		let (mut parts, body) = req.into_parts();
		let accept = Accept::from_request_parts(&mut parts, state).await.unwrap();

		let req = Request::from_parts(parts, body);

		let content_type = req
			.headers()
			.get(header::CONTENT_TYPE)
			.and_then(ContentType::from_header)
			.unwrap_or_default();

		let bytes = BytesMut::from_request(req, state)
			.await
			.map_err(|e| CodecRejection::from(e).into_codec_response(accept.into()))?;
		let data =
			Codec::from_bytes(&bytes, content_type).map_err(|e| e.into_codec_response(accept.into()))?;

		Ok(data)
	}
}

#[cfg(feature = "aide")]
impl<T> aide::operation::OperationInput for Codec<T>
where
	T: schemars::JsonSchema,
{
	fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
		axum::Json::<T>::operation_input(ctx, operation);
	}

	fn inferred_early_responses(
		ctx: &mut aide::gen::GenContext,
		operation: &mut aide::openapi::Operation,
	) -> Vec<(Option<u16>, aide::openapi::Response)> {
		axum::Json::<T>::inferred_early_responses(ctx, operation)
	}
}

#[cfg(feature = "aide")]
impl<T> aide::operation::OperationOutput for Codec<T>
where
	T: schemars::JsonSchema,
{
	type Inner = T;

	fn operation_response(
		ctx: &mut aide::gen::GenContext,
		operation: &mut aide::openapi::Operation,
	) -> Option<aide::openapi::Response> {
		axum::Json::<T>::operation_response(ctx, operation)
	}

	fn inferred_responses(
		ctx: &mut aide::gen::GenContext,
		operation: &mut aide::openapi::Operation,
	) -> Vec<(Option<u16>, aide::openapi::Response)> {
		axum::Json::<T>::inferred_responses(ctx, operation)
	}
}

#[cfg(feature = "validator")]
impl<T> validator::Validate for Codec<T>
where
	T: validator::Validate,
{
	fn validate(&self) -> Result<(), validator::ValidationErrors> {
		self.0.validate()
	}
}

/// Zero-copy codec extractor.
///
/// Similar to [`Codec`] in that it can decode from various formats,
/// but different in that the backing bytes are kept alive after decoding
/// and it cannot be used as a response encoder.
///
/// Note that the decoded data cannot be modified, as it is borrowed.
/// If you need to modify the data, use [`Codec`] instead.
///
/// # Examples
///
/// ```edition2021
/// # use axum_codec::{BorrowCodec, ContentType};
/// # use axum::response::Response;
/// # use std::borrow::Cow;
/// #
/// # fn main() {
/// #[axum_codec::apply(decode)]
/// struct Greeting<'d> {
///   hello: Cow<'d, [u8]>,
/// }
///
/// async fn my_route(body: BorrowCodec<Greeting<'_>>) -> Result<(), Response> {
///   // do something with `body.hello`...
///   println!("{:?}", body.hello);
///
///   Ok(())
/// }
/// # }
/// ```
///
/// # Errors
///
/// See [`CodecRejection`] for more information.
pub struct BorrowCodec<T> {
	data: T,
	#[allow(dead_code)]
	#[doc(hidden)]
	bytes: BytesMut,
}

impl<T> BorrowCodec<T> {
	/// Returns a mutable reference to the inner value.
	///
	/// # Safety
	///
	/// Callers must ensure that the inner value is not kept alive longer
	/// than the original [`BorrowCodec`] that it came from. For example,
	/// via [`std::mem::swap`] or [`std::mem::replace`].
	pub unsafe fn as_mut_unchecked(&mut self) -> &mut T {
		&mut self.data
	}
}

impl<T> AsRef<T> for BorrowCodec<T> {
	fn as_ref(&self) -> &T {
		self
	}
}

impl<T> Deref for BorrowCodec<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<T> fmt::Debug for BorrowCodec<T>
where
	T: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("BorrowCodec")
			.field("data", &self.data)
			.finish_non_exhaustive()
	}
}

impl<T> PartialEq for BorrowCodec<T>
where
	T: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.data == other.data
	}
}

impl<T> Eq for BorrowCodec<T> where T: Eq {}

impl<T> PartialOrd for BorrowCodec<T>
where
	T: PartialOrd,
{
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.data.partial_cmp(&other.data)
	}
}

impl<T> Ord for BorrowCodec<T>
where
	T: Ord,
{
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.data.cmp(&other.data)
	}
}

impl<T> std::hash::Hash for BorrowCodec<T>
where
	T: std::hash::Hash,
{
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.data.hash(state);
	}
}

impl<'de, T> BorrowCodec<T>
where
	T: CodecDecode<'de>,
{
	/// Creates a new [`BorrowCodec`] from the given bytes and content type.
	///
	/// # Errors
	///
	/// See [`CodecRejection`] for more information.
	pub fn from_bytes(bytes: BytesMut, content_type: ContentType) -> Result<Self, CodecRejection> {
		let data = Codec::<T>::from_bytes(
				// SAFETY: The bytes that are being referenced by the slice are behind a pointer
				// so they will not move. The bytes are also kept alive by the struct that contains
				// this struct that references the slice, so the bytes will not be deallocated
				// while this struct is alive.
				unsafe { std::slice::from_raw_parts(bytes.as_ptr(), bytes.len()) },
				content_type,
			)?
			.into_inner();

		Ok(Self {
			data,
			bytes,
		})
	}
}

#[axum::async_trait]
impl<T, S> FromRequest<S> for BorrowCodec<T>
where
	T: CodecDecode<'static>,
	S: Send + Sync + 'static,
{
	type Rejection = Response;

	async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
		let (mut parts, body) = req.into_parts();
		let accept = Accept::from_request_parts(&mut parts, state).await.unwrap();

		let req = Request::from_parts(parts, body);

		let content_type = req
			.headers()
			.get(header::CONTENT_TYPE)
			.and_then(ContentType::from_header)
			.unwrap_or_default();

		let bytes = BytesMut::from_request(req, state)
			.await
			.map_err(|e| CodecRejection::from(e).into_codec_response(accept.into()))?;

		let data =
			Self::from_bytes(bytes, content_type).map_err(|e| e.into_codec_response(accept.into()))?;

		#[cfg(feature = "validator")]
		data
			.as_ref()
			.validate()
			.map_err(|e| CodecRejection::from(e).into_codec_response(accept.into()))?;

		Ok(data)
	}
}

#[cfg(feature = "aide")]
impl<T> aide::operation::OperationInput for BorrowCodec<T>
where
	T: schemars::JsonSchema,
{
	fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
		axum::Json::<T>::operation_input(ctx, operation);
	}

	fn inferred_early_responses(
		ctx: &mut aide::gen::GenContext,
		operation: &mut aide::openapi::Operation,
	) -> Vec<(Option<u16>, aide::openapi::Response)> {
		axum::Json::<T>::inferred_early_responses(ctx, operation)
	}
}

#[cfg(test)]
mod test {
	use super::{Codec, ContentType};

	#[crate::apply(decode)]
	#[derive(Debug, PartialEq, Eq)]
	struct Data {
		hello: String,
	}

	#[test]
	fn test_json_codec() {
		let bytes = b"{\"hello\": \"world\"}";

		let Codec(data) = Codec::<Data>::from_bytes(bytes, ContentType::Json).unwrap();

		assert_eq!(data, Data {
			hello: "world".into()
		});
	}

	#[test]
	fn test_msgpack_codec() {
		let bytes = b"\x81\xa5hello\xa5world";

		let Codec(data) = Codec::<Data>::from_bytes(bytes, ContentType::MsgPack).unwrap();

		assert_eq!(data, Data {
			hello: "world".into()
		});
	}
}

#[cfg(any(test, miri))]
mod miri {
	use std::borrow::Cow;

	use bytes::Bytes;

	use super::*;

	#[crate::apply(decode, crate = "crate")]
	#[derive(Debug, PartialEq, Eq)]
	struct BorrowData<'a> {
		#[serde(borrow)]
		hello: Cow<'a, str>,
	}

	#[test]
	fn test_zero_copy() {
		let bytes = b"{\"hello\": \"world\"}".to_vec();
		let data =
			BorrowCodec::<BorrowData>::from_bytes(BytesMut::from(Bytes::from(bytes)), ContentType::Json).unwrap();

		assert_eq!(data.hello, Cow::Borrowed("world"));
	}
}
