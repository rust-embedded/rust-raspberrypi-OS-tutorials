// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2019-2022 Andre Richter <andre.o.richter@gmail.com>

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

#[proc_macro_attribute]
pub fn kernel_test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    let test_name = &format!("{}", f.sig.ident);
    let test_ident = Ident::new(
        &format!("{}_TEST_CONTAINER", f.sig.ident.to_string().to_uppercase()),
        Span::call_site(),
    );
    let test_code_block = f.block;

    quote!(
        #[test_case]
        const #test_ident: test_types::UnitTest = test_types::UnitTest {
            name: #test_name,
            test_func: || #test_code_block,
        };
    )
    .into()
}
