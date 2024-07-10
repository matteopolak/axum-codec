use std::{future::Future, pin::Pin};

use axum::{
	extract::{FromRequest, FromRequestParts, Request},
	handler::Handler,
	response::{IntoResponse, Response},
};

use crate::{macros::all_the_tuples, Accept, Codec, CodecEncode, ContentType};

pub trait IntoCodecResponse {
	fn into_codec_response(self, content_type: ContentType) -> Response;

	/// Converts the value into a response with a default content type.
	fn into_response(self) -> Response
	where
		Self: Sized,
	{
		self.into_codec_response(ContentType::default())
	}
}

pub struct CodecResponse(axum::response::Response);

impl From<Response> for CodecResponse {
	fn from(inner: Response) -> Self {
		Self(inner)
	}
}

impl From<CodecResponse> for Response {
	fn from(inner: CodecResponse) -> Self {
		inner.0
	}
}

impl CodecResponse {
	#[inline]
	#[must_use]
	pub fn into_inner(self) -> Response {
		self.0
	}
}

impl IntoResponse for CodecResponse {
	fn into_response(self) -> Response {
		self.into_inner()
	}
}

impl IntoCodecResponse for CodecResponse {
	fn into_codec_response(self, _content_type: ContentType) -> Response {
		self.into_inner()
	}
}

impl<D> IntoCodecResponse for D
where
	D: CodecEncode,
{
	fn into_codec_response(self, content_type: ContentType) -> Response {
		Codec(self).to_response(content_type)
	}
}

impl<D> IntoCodecResponse for Codec<D>
where
	D: CodecEncode,
{
	fn into_codec_response(self, content_type: ContentType) -> Response {
		self.to_response(content_type)
	}
}

#[cfg(not(feature = "aide"))]
pub trait Input {}
#[cfg(not(feature = "aide"))]
impl<T> Input for T {}

#[cfg(feature = "aide")]
pub trait Input: aide::OperationInput {}
#[cfg(feature = "aide")]
impl<T> Input for T where T: aide::OperationInput {}

pub trait CodecHandler<T, I: Input, D, S>: Clone + Send + 'static {
	type Future: Future<Output = Response> + Send;

	fn call(self, req: Request, state: S) -> Self::Future;
}

/// Transforms a function (that returns [`IntoCodecResponse`]) into a regular handler.
pub struct CodecHandlerFn<H, I, D> {
	pub(crate) f: H,
	pub(crate) _marker: std::marker::PhantomData<(I, D)>,
}

impl<H, I, D> CodecHandlerFn<H, I, D> {
	pub(crate) fn new(f: H) -> Self {
		Self {
			f,
			_marker: std::marker::PhantomData,
		}
	}
}

impl<H, I, D> Clone for CodecHandlerFn<H, I, D>
where
	H: Clone,
{
	fn clone(&self) -> Self {
		Self {
			f: self.f.clone(),
			_marker: std::marker::PhantomData,
		}
	}
}

#[cfg(feature = "aide")]
impl<H, I, D> aide::OperationInput for CodecHandlerFn<H, I, D>
where
	I: aide::OperationInput,
{
	fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
		I::operation_input(ctx, operation);
	}

	fn inferred_early_responses(
		ctx: &mut aide::gen::GenContext,
		operation: &mut aide::openapi::Operation,
	) -> Vec<(Option<u16>, aide::openapi::Response)> {
		I::inferred_early_responses(ctx, operation)
	}
}

#[cfg(feature = "aide")]
impl<H, I, D> aide::OperationOutput for CodecHandlerFn<H, I, D>
where
	Codec<D>: aide::OperationOutput,
{
	type Inner = <Codec<D> as aide::OperationOutput>::Inner;

	fn operation_response(
		ctx: &mut aide::gen::GenContext,
		operation: &mut aide::openapi::Operation,
	) -> Option<aide::openapi::Response> {
		<Codec<D> as aide::OperationOutput>::operation_response(ctx, operation)
	}

	fn inferred_responses(
		ctx: &mut aide::gen::GenContext,
		operation: &mut aide::openapi::Operation,
	) -> Vec<(Option<u16>, aide::openapi::Response)> {
		<Codec<D> as aide::OperationOutput>::inferred_responses(ctx, operation)
	}
}

#[cfg(feature = "aide")]
impl<F, I, D> aide::operation::OperationHandler<I, Codec<D>> for CodecHandlerFn<F, I, D>
where
	I: aide::OperationInput,
	Codec<D>: aide::OperationOutput,
{
}

impl<T, H, I, D, S> Handler<T, S> for CodecHandlerFn<H, I, D>
where
	H: CodecHandler<T, I, D, S>,
	S: Send + Sync + 'static,
	I: Input + Send + 'static,
	D: IntoCodecResponse + Send + 'static,
{
	type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

	#[inline]
	fn call(self, req: Request, state: S) -> Self::Future {
		Box::pin(async move { CodecHandler::<T, I, D, S>::call(self.f, req, state).await })
	}
}

impl<F, Fut, Res, S> CodecHandler<((),), (), Res, S> for F
where
	F: FnOnce() -> Fut + Clone + Send + 'static,
	Fut: Future<Output = Res> + Send,
	Res: IntoCodecResponse,
	S: Send + Sync + 'static,
{
	type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

	fn call(self, req: Request, state: S) -> Self::Future {
		Box::pin(async move {
			let (mut parts, ..) = req.into_parts();

			let content_type = match Accept::from_request_parts(&mut parts, &state).await {
				Ok(content_type) => content_type,
				Err(rejection) => return rejection.into_response(),
			};

			self().await.into_codec_response(content_type.into())
		})
	}
}

macro_rules! impl_handler {
	(
		[$($ty:ident),*], $last:ident
	) => {
		#[allow(non_snake_case, unused_mut)]
		impl<F, Fut, S, Res, M, $($ty,)* $last> CodecHandler<(M, $($ty,)* $last,), ($($ty,)* $last,), Res, S> for F
		where
			F: FnOnce($($ty,)* $last,) -> Fut + Clone + Send + 'static,
			Fut: Future<Output = Res> + Send,
			S: Send + Sync + 'static,
			Res: IntoCodecResponse,
			$( $ty: Input + FromRequestParts<S> + Send, )*
			$last: Input + FromRequest<S, M> + Send,
		{
			type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

			fn call(self, req: Request, state: S) -> Self::Future {
				Box::pin(async move {
					let (mut parts, body) = req.into_parts();

					let content_type = match Accept::from_request_parts(&mut parts, &state).await {
						Ok(content_type) => content_type,
						Err(rejection) => return rejection.into_response(),
					};

					$(
						let $ty = match $ty::from_request_parts(&mut parts, &state).await {
							Ok(value) => value,
							Err(rejection) => return rejection.into_response(),
						};
					)*

					let req = Request::from_parts(parts, body);

					let $last = match $last::from_request(req, &state).await {
						Ok(value) => value,
						Err(rejection) => return rejection.into_response(),
					};

					self($($ty,)* $last,).await
						.into_codec_response(content_type.into())
				})
			}
		}
	};
}

all_the_tuples!(impl_handler);
