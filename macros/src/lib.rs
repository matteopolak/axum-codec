#![allow(unused_variables)]
#![allow(dead_code)]

use proc_macro::TokenStream;
#[cfg(any(feature = "bincode", feature = "bitcode", feature = "serde"))]
use quote::quote;
use syn::{
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	Ident, Token,
};

struct Args {
	encode: bool,
	decode: bool,
}

impl Parse for Args {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let options = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;

		let mut encode = false;
		let mut decode = false;

		for option in options {
			match option.to_string().as_str() {
				"encode" => encode = true,
				"decode" => decode = true,
				_ => return Err(syn::Error::new(option.span(), "unknown option")),
			}
		}

		if !encode && !decode {
			return Err(syn::Error::new(
				input.span(),
				"at least one of `encode` or `decode` must be enabled",
			));
		}

		Ok(Self { encode, decode })
	}
}

/// A utility macro for automatically deriving the correct traits
/// depending on the enabled features.
#[proc_macro_attribute]
pub fn apply(attr: TokenStream, input: TokenStream) -> TokenStream {
	let args = syn::parse_macro_input!(attr as Args);

	let mut tokens = TokenStream::default();

	#[cfg(feature = "serde")]
	{
		if args.encode {
			tokens.extend(TokenStream::from(quote! {
				#[derive(axum_codec::__private::serde::Serialize)]
			}));
		}

		if args.decode {
			tokens.extend(TokenStream::from(quote! {
				#[derive(axum_codec::__private::serde::Deserialize)]
			}));
		}

		tokens.extend(TokenStream::from(quote! {
			#[serde(crate = "axum_codec::__private::serde")]
		}));
	}

	#[cfg(feature = "bincode")]
	{
		if args.encode {
			tokens.extend(TokenStream::from(quote! {
				#[derive(axum_codec::__private::bincode::Encode)]
			}));
		}

		if args.decode {
			tokens.extend(TokenStream::from(quote! {
				#[derive(axum_codec::__private::bincode::Decode)]
			}));
		}

		tokens.extend(TokenStream::from(quote! {
			#[bincode(crate = "axum_codec::__private::bincode")]
		}));
	}

	// TODO: Implement #[bitcode(crate = "...")]
	// For now, use the real crate name so the error is nicer.
	#[cfg(feature = "bitcode")]
	{
		if args.encode {
			tokens.extend(TokenStream::from(quote! {
				#[derive(bitcode::Encode)]
			}));
		}

		if args.decode {
			tokens.extend(TokenStream::from(quote! {
				#[derive(bitcode::Decode)]
			}));
		}
	}

	#[cfg(feature = "aide")]
	tokens.extend(TokenStream::from(quote! {
		#[derive(axum_codec::__private::schemars::JsonSchema)]
		#[schemars(crate = "axum_codec::__private::schemars")]
	}));

	// TODO: Implement #[validate(crate = "...")]
	// For now, use the real crate name so the error is nicer.
	#[cfg(feature = "validator")]
	if args.decode {
		tokens.extend(TokenStream::from(quote! {
			#[derive(validator::Validate)]
		}));
	}

	tokens.extend(input);
	tokens
}
