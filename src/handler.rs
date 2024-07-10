use std::{future::Future, pin::Pin};

use axum::{
	extract::{FromRequest, FromRequestParts, Request},
	handler::Handler,
	response::{IntoResponse, Response},
};

use crate::{macros::all_the_tuples, Accept, Codec, CodecEncode};

pub trait IntoCodec<D> {
	fn into_codec(self) -> Codec<D>;
}

impl<D> IntoCodec<D> for D
where
	D: CodecEncode,
{
	fn into_codec(self) -> Codec<D> {
		Codec(self)
	}
}

impl<D> IntoCodec<D> for Codec<D> {
	fn into_codec(self) -> Codec<D> {
		self
	}
}

pub trait CodecHandler<T, D, S>: Clone + Send + 'static {
	type Future: Future<Output = Response> + Send;

	fn call(self, req: Request, state: S) -> Self::Future;
}

/// Transforms a function (that returns [`IntoCodec<D>`]) into a regular handler.
pub struct CodecHandlerFn<F, D> {
	pub(crate) f: F,
	pub(crate) _marker: std::marker::PhantomData<(D,)>,
}

impl<F, D> CodecHandlerFn<F, D> {
	pub(crate) fn new(f: F) -> Self {
		Self {
			f,
			_marker: std::marker::PhantomData,
		}
	}
}

impl<F, D> Clone for CodecHandlerFn<F, D>
where
	F: Clone,
{
	fn clone(&self) -> Self {
		Self {
			f: self.f.clone(),
			_marker: std::marker::PhantomData,
		}
	}
}

impl<T, F, D, S> Handler<T, S> for CodecHandlerFn<F, D>
where
	F: CodecHandler<T, D, S>,
	S: Send + Sync + 'static,
	D: CodecEncode + Send + 'static,
{
	type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

	#[inline]
	fn call(self, req: Request, state: S) -> Self::Future {
		Box::pin(async move { CodecHandler::<T, D, S>::call(self.f, req, state).await })
	}
}

impl<D, F, Fut, Res, S> CodecHandler<((),), D, S> for F
where
	F: FnOnce() -> Fut + Clone + Send + 'static,
	Fut: Future<Output = Res> + Send,
	Res: IntoCodec<D>,
	S: Send + Sync + 'static,
	D: CodecEncode + Send + 'static,
{
	type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

	fn call(self, req: Request, state: S) -> Self::Future {
		Box::pin(async move {
			let (mut parts, ..) = req.into_parts();

			let content_type = match Accept::from_request_parts(&mut parts, &state).await {
				Ok(content_type) => content_type,
				Err(rejection) => return rejection.into_response(),
			};

			self().await.into_codec().to_response(content_type)
		})
	}
}

macro_rules! impl_handler {
	(
		[$($ty:ident),*], $last:ident
	) => {
		#[allow(non_snake_case, unused_mut)]
		impl<D, F, Fut, S, Res, M, $($ty,)* $last> CodecHandler<(M, $($ty,)* $last,), D, S> for F
		where
			F: FnOnce($($ty,)* $last,) -> Fut + Clone + Send + 'static,
			Fut: Future<Output = Res> + Send,
			S: Send + Sync + 'static,
			Res: IntoCodec<D>,
			D: CodecEncode + Send + 'static,
			$( $ty: FromRequestParts<S> + Send, )*
			$last: FromRequest<S, M> + Send,
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
						.into_codec()
						.to_response(content_type)
				})
			}
		}
	};
}

all_the_tuples!(impl_handler);
