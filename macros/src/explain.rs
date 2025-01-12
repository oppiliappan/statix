use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parse, ItemStruct, Lit};

pub fn generate_explain_impl(struct_item: &ItemStruct) -> TokenStream2 {
    let struct_name = &struct_item.ident;
    let explain = struct_item
        .attrs
        .iter()
        .filter(|attr| {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("doc") {
                    Ok(())
                } else {
                    Err(meta.error("unregognized meta attributes..."))
                }
            })
            .is_ok()
        })
        .filter_map(|attr| attr.parse_args_with(syn::Lit::parse).ok())
        .filter_map(|lit| match lit {
            Lit::Str(lit_str) => Some(lit_str.value()),
            _ => None,
        })
        .map(|s| s.trim_start().to_owned())
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
