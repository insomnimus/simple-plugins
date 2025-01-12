// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

#![allow(clippy::tabs_in_doc_comments)]

use proc_macro::TokenStream;
use syn::{
	LitFloat,
	__private::ToTokens,
	parse_quote,
	visit_mut,
	visit_mut::VisitMut,
	Expr,
	ItemFn,
	Lit,
	TypePath,
};

struct ReplaceFloat {
	type_path: TypePath,
}

impl VisitMut for ReplaceFloat {
	fn visit_expr_mut(&mut self, node: &mut Expr) {
		let type_path = &self.type_path;

		if let Expr::Lit(expr) = &node {
			if let Lit::Float(float) = &expr.lit {
				// Intentionally ignore converting literals with a literl (e.g. 2.0_f64)
				if float.suffix().is_empty() {
					let val = float.base10_parse::<f64>().unwrap();

					*node = if val == 0.0 {
						parse_quote!(#type_path::ZERO)
					} else if val == 0.5 {
						parse_quote!(#type_path::HALF)
					} else if val == 1.0 {
						parse_quote!(#type_path::ONE)
					} else {
						let unsuffixed: LitFloat = syn::parse_str(float.base10_digits()).unwrap();
						parse_quote!(#type_path::splat(#unsuffixed))
					};

					return;
				}
			}
		}

		visit_mut::visit_expr_mut(self, node);
	}
}

/** Replace float literals in a function.

/# Usage
```ignore
use components::SimdFloat;

#[replace_float_literals(T)]
fn double<T: SimdFloat>(val: T) -> T {
	val * 2.0
}
```
**/
#[proc_macro_attribute]
pub fn replace_float_literals(attr: TokenStream, item: TokenStream) -> TokenStream {
	let type_path = syn::parse::<TypePath>(attr).unwrap();
	let mut item = syn::parse::<ItemFn>(item).unwrap();
	let mut rep = ReplaceFloat { type_path };

	rep.visit_item_fn_mut(&mut item);

	item.into_token_stream().into()
}
