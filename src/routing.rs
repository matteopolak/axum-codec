use std::convert::Infallible;

use axum::routing;

use crate::{handler::CodecHandlerFn, CodecEncode, CodecHandler};

pub struct MethodRouter<S, E> {
	inner: routing::MethodRouter<S, E>,
}

impl<S, E> Clone for MethodRouter<S, E> {
	fn clone(&self) -> Self {
		Self {
			inner: self.inner.clone(),
		}
	}
}

impl<S, E> From<MethodRouter<S, E>> for routing::MethodRouter<S, E> {
	fn from(router: MethodRouter<S, E>) -> Self {
		router.inner
	}
}

impl<S, E> From<routing::MethodRouter<S, E>> for MethodRouter<S, E> {
	fn from(router: routing::MethodRouter<S, E>) -> Self {
		Self { inner: router }
	}
}

macro_rules! method_router_chain_method {
	($name:ident) => {
		#[doc = concat!("Route `", stringify!($name) ,"` requests to the given handler. See [`axum::routing::MethodRouter::", stringify!($name) , "`] for more details.")]
		#[must_use]
		pub fn $name<T, H, D>(mut self, handler: H) -> Self
		where
			H: CodecHandler<T, D, S> + Clone + Send + Sync + 'static,
			D: CodecEncode + Send + Sync + 'static,
			S: Clone + Send + Sync + 'static,
			T: 'static
		{
			self.inner = self.inner.$name(CodecHandlerFn::new(handler));
			self
		}
  };
}

impl<S> MethodRouter<S, Infallible>
where
	S: Clone + Send + Sync + 'static,
{
	method_router_chain_method!(delete);
	method_router_chain_method!(get);
	method_router_chain_method!(head);
	method_router_chain_method!(options);
	method_router_chain_method!(patch);
	method_router_chain_method!(post);
	method_router_chain_method!(put);
	method_router_chain_method!(trace);
}

macro_rules! method_router_top_level {
	($name:ident) => {
		#[doc = concat!("Route `", stringify!($name) ,"` requests to the given handler. See [`axum::routing::", stringify!($name) , "`] for more details.")]
		pub fn $name<T, H, D, S>(handler: H) -> MethodRouter<S, Infallible>
		where
			H: CodecHandler<T, D, S> + Clone + Send + Sync + 'static,
			D: CodecEncode + Send + Sync + 'static,
			S: Clone + Send + Sync + 'static,
			T: 'static
		{
			MethodRouter::from(routing::$name(CodecHandlerFn::new(handler)))
		}
	};
}

method_router_top_level!(delete);
method_router_top_level!(get);
method_router_top_level!(head);
method_router_top_level!(options);
method_router_top_level!(patch);
method_router_top_level!(post);
method_router_top_level!(put);
method_router_top_level!(trace);
