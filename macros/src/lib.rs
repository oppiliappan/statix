use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_macro_input,
    punctuated::Punctuated,
    Ident, ItemStruct, Lit, Path, Token,
};

struct KeyValue {
    key: Ident,
    _eq: Token![=],
    value: Lit,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(Self {
            key: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

struct LintMeta(HashMap<Ident, Lit>);

impl Parse for LintMeta {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        Ok(Self(
            Punctuated::<KeyValue, Token![,]>::parse_terminated(input)?
                .into_iter()
                .map(|item| (item.key, item.value))
                .collect(),
        ))
    }
}

fn generate_self_impl(struct_name: &Ident) -> TokenStream2 {
    quote! {
        impl #struct_name {
            pub fn new() -> Box<Self> {
                Box::new(Self)
            }
        }
    }
}

fn generate_meta_impl(struct_name: &Ident, meta: &LintMeta) -> TokenStream2 {
    let name_fn = generate_name_fn(meta);
    let note_fn = generate_note_fn(meta);
    let match_with_fn = generate_match_with_fn(meta);
    quote! {
        impl Metadata for #struct_name {
            #name_fn
            #note_fn
            #match_with_fn
        }
    }
}

fn generate_name_fn(meta: &LintMeta) -> TokenStream2 {
    let name = meta
        .0
        .get(&format_ident!("name"))
        .unwrap_or_else(|| panic!("`name` not present"));
    quote! {
        fn name(&self) -> &str {
            #name
        }
    }
}

fn generate_note_fn(meta: &LintMeta) -> TokenStream2 {
    let note = meta
        .0
        .get(&format_ident!("note"))
        .unwrap_or_else(|| panic!("`note` not present"));
    quote! {
        fn note(&self) -> &str {
            #note
        }
    }
}

fn generate_match_with_fn(meta: &LintMeta) -> TokenStream2 {
    let match_with_lit = meta
        .0
        .get(&format_ident!("match_with"))
        .unwrap_or_else(|| panic!("`match_with` not present"));
    if let Lit::Str(match_with) = match_with_lit {
        let path: Path = match_with
            .parse()
            .ok()
            .unwrap_or_else(|| panic!("`match_with` does not contain valid path"));
        quote! {
            fn match_with(&self, with: &SyntaxKind) -> bool {
                *with == #path
            }
        }
    } else {
        panic!("`match_with` has non-literal value")
    }
}

#[proc_macro_attribute]
pub fn lint(attr: TokenStream, item: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(item as ItemStruct);
    let meta = parse_macro_input!(attr as LintMeta);

    let struct_name = &struct_item.ident;
    let self_impl = generate_self_impl(struct_name);
    let meta_impl = generate_meta_impl(struct_name, &meta);
    (quote! {
        #struct_item

        ::lazy_static::lazy_static! {
            pub static ref LINT: Box<dyn crate::Lint> = #struct_name::new();
        }

        #self_impl
        #meta_impl

        impl Lint for #struct_name {}
    })
    .into()
}
