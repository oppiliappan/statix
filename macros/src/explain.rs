use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemStruct, Lit, Meta, MetaNameValue};

pub fn generate_explain_impl(struct_item: &ItemStruct) -> TokenStream2 {
    let struct_name = &struct_item.ident;
    let explain = struct_item
        .attrs
        .iter()
        .filter_map(|attr| match attr.parse_meta().ok() {
            Some(Meta::NameValue(MetaNameValue {
                path,
                lit: Lit::Str(str_lit),
                ..
            })) if path.is_ident("doc") => Some(str_lit.value()),
            _ => None,
        })
        .map(|s| s.strip_prefix(' ').unwrap_or(&s).to_owned())
        .collect::<Vec<_>>()
        .join("\n");
    quote! {
        impl crate::Explain for #struct_name {
            fn explanation(&self) -> &'static str {
                #explain
            }
        }
    }
}
