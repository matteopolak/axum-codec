// This is copied from Axum under the following license:
//
// Copyright 2021 Axum Contributors
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use quote::ToTokens;
use syn::{
	parse::{Parse, ParseStream},
	Token,
};

pub(crate) fn parse_assignment_attribute<K, T>(
	input: ParseStream,
	out: &mut Option<(K, T)>,
) -> syn::Result<()>
where
	K: Parse + ToTokens,
	T: Parse,
{
	let kw = input.parse()?;
	input.parse::<Token![=]>()?;
	let inner = input.parse()?;

	if out.is_some() {
		let kw_name = std::any::type_name::<K>().split("::").last().unwrap();
		let msg = format!("`{kw_name}` specified more than once");
		return Err(syn::Error::new_spanned(kw, msg));
	}

	*out = Some((kw, inner));

	Ok(())
}

pub(crate) fn second<T, K>(tuple: (T, K)) -> K {
	tuple.1
}
