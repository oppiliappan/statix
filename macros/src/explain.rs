use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemStruct, Lit, Meta, MetaNameValue};

pub fn generate_explain_impl(struct_item: &ItemStruct) -> TokenStream2 {
    let struct_name = &struct_item.ident;
    let explain = struct_item
        .attrs
        .iter()
        .filter_map(|a| a.parse_meta().ok())
        .filter_map(|meta| match meta {
            Meta::NameValue(MetaNameValue { path, lit, .. }) if path.is_ident("doc") => Some(lit),
            _ => None,
        })
        .filter_map(|lit| match lit {
            Lit::Str(str_lit) => Some(str_lit.value()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    quote! {
        impl crate::Explain for #struct_name {
            fn explaination(&self) -> &'static str {
                #explain
            }
        }
    }
}
