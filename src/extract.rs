use axum::{
	extract::FromRequestParts,
	http::{header, request::Parts},
};

use crate::{CodecRejection, ContentType};

/// Extractor for the request's desired response [`ContentType`].
///
/// # Examples
///
/// ```edition2021
/// # use axum_codec::{Accept, Codec};
/// # use axum::{http::HeaderValue, response::IntoResponse};
/// # use serde::Serialize;
/// #
/// #[axum_codec::derive(encode)]
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
			.unwrap_or_default();

		Ok(Self(header))
	}
}
