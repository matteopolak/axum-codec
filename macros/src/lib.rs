#![cfg_attr(
	not(any(
		feature = "bincode",
		feature = "bitcode",
		feature = "serde",
		feature = "aide",
		feature = "validator"
	)),
	allow(unused_variables, dead_code)
)]

use proc_macro::TokenStream;
use syn::parse::Parse;

mod apply;
mod attr_parsing;
mod debug_handler;
mod with_position;

/// A utility macro for automatically deriving the correct traits
/// depending on the enabled features.
#[proc_macro_attribute]
pub fn apply(
	attr: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	apply::apply(attr, input)
}

/// Generates better error messages when applied to handler functions.
///
/// For more information, see [`axum::debug_handler`](https://docs.rs/axum/latest/axum/attr.debug_handler.html).
#[proc_macro_attribute]
pub fn debug_handler(_attr: TokenStream, input: TokenStream) -> TokenStream {
	#[cfg(not(debug_assertions))]
	return input;

	#[cfg(debug_assertions)]
	return expand_attr_with(_attr, input, |attrs, item_fn| {
		debug_handler::expand(attrs, item_fn, debug_handler::FunctionKind::Handler)
	});
}

/// Generates better error messages when applied to middleware functions.
///
/// For more information, see [`axum::debug_middleware`](https://docs.rs/axum/latest/axum/attr.debug_middleware.html).
#[proc_macro_attribute]
pub fn debug_middleware(_attr: TokenStream, input: TokenStream) -> TokenStream {
	#[cfg(not(debug_assertions))]
	return input;

	#[cfg(debug_assertions)]
	return expand_attr_with(_attr, input, |attrs, item_fn| {
		debug_handler::expand(attrs, item_fn, debug_handler::FunctionKind::Middleware)
	});
}

fn expand_attr_with<F, A, I, K>(attr: TokenStream, input: TokenStream, f: F) -> TokenStream
where
	F: FnOnce(A, I) -> K,
	A: Parse,
	I: Parse,
	K: quote::ToTokens,
{
	let expand_result = (|| {
		let attr = syn::parse(attr)?;
		let input = syn::parse(input)?;
		Ok(f(attr, input))
	})();
	expand(expand_result)
}

fn expand<T>(result: syn::Result<T>) -> TokenStream
where
	T: quote::ToTokens,
{
	match result {
		Ok(tokens) => {
			let tokens = (quote::quote! { #tokens }).into();
			if std::env::var_os("AXUM_MACROS_DEBUG").is_some() {
				eprintln!("{tokens}");
			}
			tokens
		}
		Err(err) => err.into_compile_error().into(),
	}
}

fn infer_state_types<'a, I>(types: I) -> impl Iterator<Item = syn::Type> + 'a
where
	I: Iterator<Item = &'a syn::Type> + 'a,
{
	types
		.filter_map(|ty| {
			if let syn::Type::Path(path) = ty {
				Some(&path.path)
			} else {
				None
			}
		})
		.filter_map(|path| {
			if let Some(last_segment) = path.segments.last() {
				if last_segment.ident != "State" {
					return None;
				}

				match &last_segment.arguments {
					syn::PathArguments::AngleBracketed(args) if args.args.len() == 1 => {
						Some(args.args.first().unwrap())
					}
					_ => None,
				}
			} else {
				None
			}
		})
		.filter_map(|generic_arg| {
			if let syn::GenericArgument::Type(ty) = generic_arg {
				Some(ty)
			} else {
				None
			}
		})
		.cloned()
}

#[doc(hidden)]
#[proc_macro]
pub fn __private_decode_trait(input: TokenStream) -> TokenStream {
	__private::decode_trait(input.into()).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn __private_encode_trait(input: TokenStream) -> TokenStream {
	__private::encode_trait(input.into()).into()
}

#[allow(unused_imports, unused_mut)]
mod __private {
	use proc_macro2::TokenStream;
	use quote::quote;

	pub fn decode_trait(input: TokenStream) -> TokenStream {
		let mut codec_trait = TokenStream::default();
		let mut codec_impl = TokenStream::default();

		codec_trait.extend(quote! {
			#input
			pub trait CodecDecode
		});

		codec_impl.extend(quote! {
			impl<T> CodecDecode for T
		});

		#[cfg(any(
			feature = "bincode",
			feature = "bitcode",
			feature = "serde",
			feature = "aide",
			feature = "validator"
		))]
		{
			codec_trait.extend(quote! {
				:
			});

			codec_impl.extend(quote! {
				where T:
			});
		}

		let mut constraints = TokenStream::default();

		#[cfg(feature = "serde")]
		{
			if !constraints.is_empty() {
				constraints.extend(quote! { + });
			}

			constraints.extend(quote! {
				serde::de::DeserializeOwned
			});
		}

		#[cfg(feature = "bincode")]
		{
			if !constraints.is_empty() {
				constraints.extend(quote! { + });
			}

			constraints.extend(quote! {
				bincode::Decode
			});
		}

		#[cfg(feature = "bitcode")]
		{
			if !constraints.is_empty() {
				constraints.extend(quote! { + });
			}

			constraints.extend(quote! {
				bitcode::DecodeOwned
			});
		}

		#[cfg(feature = "validator")]
		{
			if !constraints.is_empty() {
				constraints.extend(quote! { + });
			}

			constraints.extend(quote! {
				validator::Validate
			});
		}

		codec_trait.extend(constraints.clone());
		codec_impl.extend(constraints);

		codec_trait.extend(quote!({}));
		codec_impl.extend(quote!({}));

		codec_trait.extend(codec_impl);
		codec_trait
	}

	pub fn encode_trait(input: TokenStream) -> TokenStream {
		let mut codec_trait = TokenStream::default();
		let mut codec_impl = TokenStream::default();

		codec_trait.extend(quote! {
			#input
			pub trait CodecEncode
		});

		codec_impl.extend(quote! {
			impl<T> CodecEncode for T
		});

		#[cfg(any(
			feature = "bincode",
			feature = "bitcode",
			feature = "serde",
			feature = "aide",
			feature = "validator"
		))]
		{
			codec_trait.extend(quote! {
				:
			});

			codec_impl.extend(quote! {
				where T:
			});
		}

		let mut constraints = TokenStream::default();

		#[cfg(feature = "serde")]
		{
			if !constraints.is_empty() {
				constraints.extend(quote! { + });
			}

			constraints.extend(quote! {
				serde::Serialize
			});
		}

		#[cfg(feature = "bincode")]
		{
			if !constraints.is_empty() {
				constraints.extend(quote! { + });
			}

			constraints.extend(quote! {
				bincode::Encode
			});
		}

		#[cfg(feature = "bitcode")]
		{
			if !constraints.is_empty() {
				constraints.extend(quote! { + });
			}

			constraints.extend(quote! {
				bitcode::Encode
			});
		}

		codec_trait.extend(constraints.clone());
		codec_impl.extend(constraints);

		codec_trait.extend(quote!({}));
		codec_impl.extend(quote!({}));

		codec_trait.extend(codec_impl);
		codec_trait
	}
}
