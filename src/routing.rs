use std::convert::Infallible;

use axum::routing;

use crate::{
	handler::{CodecHandlerFn, Input},
	CodecHandler, IntoCodecResponse,
};

/// A light wrapper around axum's [`MethodRouter`](axum::routing::MethodRouter)
/// (or [`ApiMethodRouter`](aide::axum::routing::ApiMethodRouter) if the `aide`
/// feature is enabled).
///
/// However, responses are expected to be [`IntoCodecResponse`] (instead of
/// [`IntoResponse`](axum::response::IntoResponse)), as they are automatically
/// converted to the appropriate response type when appropriate.
pub struct MethodRouter<S = (), E = Infallible> {
	#[cfg(not(feature = "aide"))]
	inner: routing::MethodRouter<S, E>,
	#[cfg(feature = "aide")]
	inner: aide::axum::routing::ApiMethodRouter<S, E>,
}

impl<S, E> Clone for MethodRouter<S, E> {
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}
}

#[cfg(not(feature = "aide"))]
impl<S, E> From<MethodRouter<S, E>> for routing::MethodRouter<S, E> {
	fn from(router: MethodRouter<S, E>) -> Self {
		router.inner
	}
}

#[cfg(not(feature = "aide"))]
impl<S, E> From<routing::MethodRouter<S, E>> for MethodRouter<S, E> {
	fn from(router: routing::MethodRouter<S, E>) -> Self {
		Self { inner: router }
	}
}

#[cfg(feature = "aide")]
impl<S, E> From<routing::MethodRouter<S, E>> for MethodRouter<S, E> {
	fn from(router: routing::MethodRouter<S, E>) -> Self {
		Self {
			inner: router.into(),
		}
	}
}

#[cfg(feature = "aide")]
impl<S, E> From<MethodRouter<S, E>> for routing::MethodRouter<S, E> {
	fn from(router: MethodRouter<S, E>) -> Self {
		router.inner.into()
	}
}

#[cfg(feature = "aide")]
impl<S, E> From<aide::axum::routing::ApiMethodRouter<S, E>> for MethodRouter<S, E> {
	fn from(router: aide::axum::routing::ApiMethodRouter<S, E>) -> Self {
		Self { inner: router }
	}
}

#[cfg(feature = "aide")]
impl<S, E> From<MethodRouter<S, E>> for aide::axum::routing::ApiMethodRouter<S, E> {
	fn from(router: MethodRouter<S, E>) -> Self {
		router.inner
	}
}

#[cfg(not(feature = "aide"))]
macro_rules! method_router_chain_method {
	($name:ident, $with:ident) => {
		#[doc = concat!("Route `", stringify!($name) ,"` requests to the given handler. See [`axum::routing::MethodRouter::", stringify!($name) , "`] for more details.")]
		#[must_use]
		pub fn $name<T, H, I, D>(mut self, handler: H) -> Self
		where
			H: CodecHandler<T, I, D, S> + Clone + Send + Sync + 'static,
			I: Input + Send + 'static,
			D: IntoCodecResponse + Send + Sync + 'static,
			S: Clone + Send + Sync + 'static,
			T: 'static
		{
			self.inner = self.inner.$name(CodecHandlerFn::new(handler));
			self
		}
  };
}

#[cfg(feature = "aide")]
macro_rules! method_router_chain_method {
	($name:ident, $with:ident) => {
		#[doc = concat!("Route `", stringify!($name) ,"` requests to the given handler. See [`axum::routing::MethodRouter::", stringify!($name) , "`] for more details.")]
		#[must_use]
		pub fn $name<T, H, I, D>(mut self, handler: H) -> Self
		where
			H: CodecHandler<T, I, D, S> + Clone + Send + Sync + 'static,
			I: Input + Send + 'static,
			D: IntoCodecResponse + Send + 'static,
			S: Clone + Send + Sync + 'static,
			T: 'static,
		{
			self.inner = self.inner.$name(CodecHandlerFn::<H, I, D>::new(handler));
			self
		}

		#[doc = concat!("Route `", stringify!($name) ,"` requests to the given handler. See [`axum::routing::MethodRouter::", stringify!($name) , "`] for more details.")]
		#[must_use]
		pub fn $with<T, H, I, D, F>(mut self, handler: H, transform: F) -> Self
		where
			H: CodecHandler<T, I, D, S> + Clone + Send + Sync + 'static,
			I: Input + Send + 'static,
			D: IntoCodecResponse + Send + 'static,
			S: Clone + Send + Sync + 'static,
			T: 'static,
			F: FnOnce(aide::transform::TransformOperation) -> aide::transform::TransformOperation,
		{
			self.inner = self.inner.$with(CodecHandlerFn::<H, I, D>::new(handler), transform);
			self
		}
  };
}

impl<S> MethodRouter<S, Infallible>
where
	S: Clone + Send + Sync + 'static,
{
	method_router_chain_method!(delete, delete_with);

	method_router_chain_method!(get, get_with);

	method_router_chain_method!(head, head_with);

	method_router_chain_method!(options, options_with);

	method_router_chain_method!(patch, patch_with);

	method_router_chain_method!(post, post_with);

	method_router_chain_method!(put, put_with);

	method_router_chain_method!(trace, trace_with);
}

#[cfg(not(feature = "aide"))]
macro_rules! method_router_top_level {
	($name:ident, $with:ident) => {
		#[doc = concat!("Route `", stringify!($name) ,"` requests to the given handler. See [`axum::routing::", stringify!($name) , "`] for more details.")]
		pub fn $name<T, H, I, D, S>(handler: H) -> MethodRouter<S, Infallible>
		where
			H: CodecHandler<T, I, D, S> + Clone + Send + Sync + 'static,
			I: Input + Send + 'static,
			D: IntoCodecResponse + Send + Sync + 'static,
			S: Clone + Send + Sync + 'static,
			T: 'static
		{
			MethodRouter::from(routing::$name(CodecHandlerFn::new(handler)))
		}
	};
}

#[cfg(feature = "aide")]
macro_rules! method_router_top_level {
	($name:ident, $with:ident) => {
		#[doc = concat!("Route `", stringify!($name) ,"` requests to the given handler. See [`axum::routing::", stringify!($name) , "`] for more details.")]
		pub fn $name<T, H, I, D, S>(handler: H) -> MethodRouter<S, Infallible>
		where
			H: CodecHandler<T, I, D, S> + Clone + Send + Sync + 'static,
			I: Input + Send + 'static,
			D: IntoCodecResponse + Send + 'static,
			S: Clone + Send + Sync + 'static,
			T: 'static,
		{
			MethodRouter::from(aide::axum::routing::$name(
				CodecHandlerFn::<H, I, D>::new(handler),
			))
		}

		#[doc = concat!("Route `", stringify!($name) ,"` requests to the given handler. See [`axum::routing::", stringify!($name) , "`] for more details.")]
		#[must_use]
		pub fn $with<T, H, I, D, S, F>(handler: H, transform: F) -> MethodRouter<S, Infallible>
		where
			H: CodecHandler<T, I, D, S> + Clone + Send + Sync + 'static,
			I: Input + Send + 'static,
			D: IntoCodecResponse + Send + Sync + 'static,
			S: Clone + Send + Sync + 'static,
			T: 'static,
			F: FnOnce(aide::transform::TransformOperation) -> aide::transform::TransformOperation,
		{
			MethodRouter::from(aide::axum::routing::$with(CodecHandlerFn::<H, I, D>::new(handler), transform))
		}
	};
}

method_router_top_level!(delete, delete_with);
method_router_top_level!(get, get_with);
method_router_top_level!(head, head_with);
method_router_top_level!(options, options_with);
method_router_top_level!(patch, patch_with);
method_router_top_level!(post, post_with);
method_router_top_level!(put, put_with);
method_router_top_level!(trace, trace_with);
