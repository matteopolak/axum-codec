/// A utility macro for automatically deriving the correct traits
/// depending on the enabled features.
#[cfg(any(
	feature = "bincode",
	feature = "bitcode",
	feature = "serde",
	feature = "aide",
	feature = "validator"
))]
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
	use quote::{quote, ToTokens};

	use syn::{
		parse::{Parse, ParseStream},
		punctuated::Punctuated,
		spanned::Spanned,
		Meta, Path, Token,
	};

	struct Args {
		encode: bool,
		decode: bool,
		crate_name: Path,
	}

	impl Parse for Args {
		fn parse(input: ParseStream) -> syn::Result<Self> {
			let options = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

			let mut encode = false;
			let mut decode = false;
			let mut crate_name = syn::parse_str("axum_codec").expect("failed to parse crate name");

			for meta in options {
				match meta {
					Meta::List(list) => {
						return Err(syn::Error::new(
							list.span(),
							"expected `encode`, `decode`, or `crate`",
						))
					}
					Meta::Path(path) => {
						let ident = path.get_ident().map(|ident| ident.to_string());
						match ident.as_deref() {
							Some("encode") if encode => {
								return Err(syn::Error::new(
									path.span(),
									"option `encode` is already enabled",
								))
							}
							Some("decode") if decode => {
								return Err(syn::Error::new(
									path.span(),
									"option `decode` is already enabled",
								))
							}
							Some("encode") => encode = true,
							Some("decode") => decode = true,
							Some(other) => {
								return Err(syn::Error::new(
									path.span(),
									format!(
										"unknown option `{other}`, expected `encode` or `decode`"
									),
								))
							}
							None => {
								return Err(syn::Error::new(
									path.span(),
									"expected `encode` or `decode`",
								))
							}
						}
					}
					Meta::NameValue(name_value) => {
						if !name_value.path.is_ident("crate") {
							return Err(syn::Error::new(
								name_value.path.span(),
								"expected `crate`",
							));
						}

						let path = match name_value.value {
							syn::Expr::Lit(ref lit) => match &lit.lit {
								syn::Lit::Str(path) => path,
								_ => return Err(syn::Error::new(lit.span(), "expected a string")),
							},
							_ => {
								return Err(syn::Error::new(
									name_value.value.span(),
									"expected a literal string",
								))
							}
						};

						let mut path =
							syn::parse_str::<Path>(&path.value()).expect("failed to parse path");

						path.leading_colon = if path.is_ident("crate") {
							None
						} else {
							Some(Token![::](name_value.value.span()))
						};

						crate_name = path;
					}
				}
			}

			if !encode && !decode {
				return Err(syn::Error::new(
					input.span(),
					"at least one of `encode` or `decode` must be enabled",
				));
			}

			Ok(Self {
				encode,
				decode,
				crate_name,
			})
		}
	}

	pub fn apply(
		attr: proc_macro::TokenStream,
		input: proc_macro::TokenStream,
	) -> proc_macro::TokenStream {
		let args = syn::parse_macro_input!(attr as Args);

		let crate_name = &args.crate_name;
		let mut tokens = TokenStream::default();

		#[cfg(feature = "serde")]
		{
			if args.encode {
				tokens.extend(quote! {
					#[derive(#crate_name::__private::serde::Serialize)]
				});
			}

			if args.decode {
				tokens.extend(quote! {
					#[derive(#crate_name::__private::serde::Deserialize)]
				});
			}

			let crate_ = format!("{}::__private::serde", crate_name.to_token_stream());

			tokens.extend(quote! {
				#[serde(crate = #crate_)]
			});
		}

		#[cfg(feature = "bincode")]
		{
			if args.encode {
				tokens.extend(quote! {
					#[derive(#crate_name::__private::bincode::Encode)]
				});
			}

			if args.decode {
				tokens.extend(quote! {
					#[derive(#crate_name::__private::bincode::Decode)]
				});
			}

			let crate_ = format!("{}::__private::bincode", crate_name.to_token_stream());

			tokens.extend(quote! {
				#[bincode(crate = #crate_)]
			});
		}

		#[cfg(feature = "bitcode")]
		{
			if args.encode {
				tokens.extend(quote! {
					#[derive(#crate_name::__private::bitcode::Encode)]
				});
			}

			if args.decode {
				tokens.extend(quote! {
					#[derive(#crate_name::__private::bitcode::Decode)]
				});
			}

			let crate_ = format!("{}::__private::bitcode", crate_name.to_token_stream());

			tokens.extend(quote! {
				#[bitcode(crate = #crate_)]
			});
		}

		#[cfg(feature = "aide")]
		{
			let crate_ = format!("{}::__private::schemars", crate_name.to_token_stream());

			tokens.extend(quote! {
				#[derive(#crate_name::__private::schemars::JsonSchema)]
				#[schemars(crate = #crate_)]
			});
		}

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
