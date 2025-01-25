mod explain;
mod metadata;

use explain::generate_explain_impl;
use metadata::{generate_meta_impl, RawLintMeta};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemStruct};

fn generate_self_impl(struct_name: &Ident) -> TokenStream2 {
    quote! {
        impl #struct_name {
            pub fn new() -> Box<Self> {
                Box::new(Self)
            }
        }
    }
}

#[proc_macro_attribute]
pub fn lint(attr: TokenStream, item: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(item as ItemStruct);
    let meta = parse_macro_input!(attr as RawLintMeta);

    let struct_name = &struct_item.ident;
    let self_impl = generate_self_impl(struct_name);
    let meta_impl = generate_meta_impl(struct_name, &meta);
    let explain_impl = generate_explain_impl(&struct_item);

    (quote! {
        pub(crate) struct #struct_name;

        #self_impl
        #meta_impl
        #explain_impl

        impl crate::Lint for #struct_name {}
    })
    .into()
}
