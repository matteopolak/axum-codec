use std::{future::Future, pin::Pin};

use axum::{
	extract::{FromRequest, FromRequestParts, Request},
	handler::Handler,
	response::{IntoResponse, Response},
};

use crate::{Accept, IntoCodecResponse};

#[cfg(not(feature = "aide"))]
pub trait Input {}
#[cfg(not(feature = "aide"))]
impl<T> Input for T {}

#[cfg(feature = "aide")]
pub trait Input: aide::OperationInput {}
#[cfg(feature = "aide")]
impl<T> Input for T where T: aide::OperationInput {}

#[diagnostic::on_unimplemented(
	note = "Consider wrapping the return value in `Codec` if appropriate",
	note = "Consider using `#[axum_codec::debug_handler]` to improve the error message"
)]
pub trait CodecHandler<T, I: Input, D, S>: Clone + Send + 'static {
	type Future: Future<Output = Response> + Send;

	fn call(self, req: Request, state: S) -> Self::Future;
}

/// Transforms a function (that returns [`IntoCodecResponse`]) into a regular
/// handler.
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
	D: aide::OperationOutput,
{
	type Inner = D;

	fn operation_response(
		ctx: &mut aide::gen::GenContext,
		operation: &mut aide::openapi::Operation,
	) -> Option<aide::openapi::Response> {
		D::operation_response(ctx, operation)
	}

	fn inferred_responses(
		ctx: &mut aide::gen::GenContext,
		operation: &mut aide::openapi::Operation,
	) -> Vec<(Option<u16>, aide::openapi::Response)> {
		D::inferred_responses(ctx, operation)
	}
}

#[cfg(feature = "aide")]
impl<F, I, D> aide::operation::OperationHandler<I, crate::Codec<D>> for CodecHandlerFn<F, I, D>
where
	I: aide::OperationInput,
	D: schemars::JsonSchema,
{
}

#[cfg(feature = "aide")]
impl<F, I, D> aide::operation::OperationHandler<I, D> for CodecHandlerFn<F, I, D>
where
	I: aide::OperationInput,
	D: aide::OperationOutput,
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

			let content_type = Accept::from_request_parts(&mut parts, &state)
				.await
				.unwrap();

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

					let content_type = Accept::from_request_parts(&mut parts, &state).await.unwrap();

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

macro_rules! all_the_tuples {
	($name:ident) => {
		$name!([], T1);
		$name!([T1], T2);
		$name!([T1, T2], T3);
		$name!([T1, T2, T3], T4);
		$name!([T1, T2, T3, T4], T5);
		$name!([T1, T2, T3, T4, T5], T6);
		$name!([T1, T2, T3, T4, T5, T6], T7);
		$name!([T1, T2, T3, T4, T5, T6, T7], T8);
		$name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
		$name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
		$name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
		$name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
		$name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
		$name!(
			[T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13],
			T14
		);
		$name!(
			[T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14],
			T15
		);
		$name!(
			[T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15],
			T16
		);
	};
}

all_the_tuples!(impl_handler);
