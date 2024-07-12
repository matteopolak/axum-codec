#[cfg(not(any(
	feature = "bincode",
	feature = "bitcode",
	feature = "serde",
	feature = "aide",
	feature = "validator"
)))]
mod apply {
	use proc_macro::TokenStream;

	pub fn apply(_attr: TokenStream, input: TokenStream) -> TokenStream {
		input
	}
}

/// A utility macro for automatically deriving the correct traits
/// depending on the enabled features.
#[proc_macro_attribute]
pub fn apply(
	attr: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	apply::apply(attr, input)
}

#[cfg(any(
	feature = "bincode",
	feature = "bitcode",
	feature = "serde",
	feature = "aide",
	feature = "validator"
))]
mod apply {
	use proc_macro2::TokenStream;
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
					"encode" if encode => {
						return Err(syn::Error::new(
							option.span(),
							"option `encode` is already enabled",
						))
					}
					"decode" if decode => {
						return Err(syn::Error::new(
							option.span(),
							"option `decode` is already enabled",
						))
					}
					"encode" => encode = true,
					"decode" => decode = true,
					other => {
						return Err(syn::Error::new(
							option.span(),
							format!("unknown option `{other}`, expected `encode` or `decode`"),
						))
					}
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

	pub fn apply(
		attr: proc_macro::TokenStream,
		input: proc_macro::TokenStream,
	) -> proc_macro::TokenStream {
		let args = syn::parse_macro_input!(attr as Args);

		let mut tokens = TokenStream::default();

		#[cfg(feature = "serde")]
		{
			if args.encode {
				tokens.extend(quote! {
					#[derive(axum_codec::__private::serde::Serialize)]
				});
			}

			if args.decode {
				tokens.extend(quote! {
					#[derive(axum_codec::__private::serde::Deserialize)]
				});
			}

			tokens.extend(quote! {
				#[serde(crate = "axum_codec::__private::serde")]
			});
		}

		#[cfg(feature = "bincode")]
		{
			if args.encode {
				tokens.extend(quote! {
					#[derive(axum_codec::__private::bincode::Encode)]
				});
			}

			if args.decode {
				tokens.extend(quote! {
					#[derive(axum_codec::__private::bincode::Decode)]
				});
			}

			tokens.extend(quote! {
				#[bincode(crate = "axum_codec::__private::bincode")]
			});
		}

		// TODO: Implement #[bitcode(crate = "...")]
		// For now, use the real crate name so the error is nicer.
		#[cfg(feature = "bitcode")]
		{
			if args.encode {
				tokens.extend(quote! {
					#[derive(bitcode::Encode)]
				});
			}

			if args.decode {
				tokens.extend(quote! {
					#[derive(bitcode::Decode)]
				});
			}
		}

		#[cfg(feature = "aide")]
		tokens.extend(quote! {
			#[derive(axum_codec::__private::schemars::JsonSchema)]
			#[schemars(crate = "axum_codec::__private::schemars")]
		});

		// TODO: Implement #[validate(crate = "...")]
		// For now, use the real crate name so the error is nicer.
		#[cfg(feature = "validator")]
		if args.decode {
			tokens.extend(quote! {
				#[derive(validator::Validate)]
			});
		}

		tokens.extend(TokenStream::from(input));
		tokens.into()
	}
}

#[doc(hidden)]
#[proc_macro]
pub fn __private_decode_trait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	__private::decode_trait(input.into()).into()
}

#[doc(hidden)]
#[proc_macro]
pub fn __private_encode_trait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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

		#[cfg(feature = "aide")]
		{
			if !constraints.is_empty() {
				constraints.extend(quote! { + });
			}

			constraints.extend(quote! {
				schemars::JsonSchema
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

		#[cfg(feature = "aide")]
		{
			if !constraints.is_empty() {
				constraints.extend(quote! { + });
			}

			constraints.extend(quote! {
				schemars::JsonSchema
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
