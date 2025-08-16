use std::collections::HashMap;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Expr, ExprArray, Ident, Lit, Path, Token,
};

struct KeyValue {
    key: Ident,
    _eq: Token![=],
    value: Expr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            key: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct RawLintMeta(HashMap<Ident, Expr>);

impl Parse for RawLintMeta {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self(
            Punctuated::<KeyValue, Token![,]>::parse_terminated(input)?
                .into_iter()
                .map(|item| (item.key, item.value))
                .collect(),
        ))
    }
}

pub struct LintMeta<'μ> {
    name: &'μ Lit,
    note: &'μ Lit,
    code: &'μ Lit,
    match_with: MatchWith<'μ>,
}

enum MatchWith<'π> {
    Path(&'π Path),
    Array(&'π ExprArray),
}

fn extract<'λ>(id: &str, raw: &'λ RawLintMeta) -> &'λ Expr {
    raw.0
        .get(&format_ident!("{}", id))
        .unwrap_or_else(|| panic!("`{}` not present", id))
}

fn as_lit(e: &Expr) -> &Lit {
    match e {
        Expr::Lit(l) => &l.lit,
        _ => panic!("expected a literal"),
    }
}

impl<'μ> LintMeta<'μ> {
    fn from_raw(raw: &'μ RawLintMeta) -> Self {
        let name = as_lit(extract("name", raw));
        let note = as_lit(extract("note", raw));
        let code = as_lit(extract("code", raw));
        let match_with_expr = extract("match_with", raw);
        let match_with = match match_with_expr {
            Expr::Path(p) => MatchWith::Path(&p.path),
            Expr::Array(a) => MatchWith::Array(a),
            _ => panic!("`match_with` is neither a path nor an array"),
        };
        Self {
            name,
            note,
            code,
            match_with,
        }
    }

    fn generate_name_fn(&self) -> TokenStream2 {
        let name_str = self.name;
        quote! {
            fn name(&self) -> &'static str {
                #name_str
            }
        }
    }

    fn generate_note_fn(&self) -> TokenStream2 {
        let note_str = self.note;
        quote! {
            fn note(&self) -> &'static str {
                #note_str
            }
        }
    }

    fn generate_code_fn(&self) -> TokenStream2 {
        let code_int = self.code;
        quote! {
            fn code(&self) -> u32 {
                #code_int
            }
        }
    }

    fn generate_match_with_fn(&self) -> TokenStream2 {
        match self.match_with {
            MatchWith::Path(p) => {
                quote! {
                    fn match_with(&self, with: &SyntaxKind) -> bool {
                        #p == *with
                    }
                }
            }
            MatchWith::Array(a) => {
                quote! {
                    fn match_with(&self, with: &SyntaxKind) -> bool {
                        #a.contains(with)
                    }
                }
            }
        }
    }

    fn generate_match_kind_fn(&self) -> TokenStream2 {
        match self.match_with {
            MatchWith::Path(p) => {
                quote! {
                    fn match_kind(&self) -> Vec<SyntaxKind> {
                        vec![#p]
                    }
                }
            }
            MatchWith::Array(a) => {
                quote! {
                    fn match_kind(&self) -> Vec<SyntaxKind> {
                        #a.to_vec()
                    }
                }
            }
        }
    }

    fn generate_report_fn() -> TokenStream2 {
        quote! {
            fn report(&self) -> crate::Report {
                crate::Report::new(self.note(), self.code())
            }
        }
    }
}

pub fn generate_meta_impl(struct_name: &Ident, meta: &RawLintMeta) -> TokenStream2 {
    let not_raw = LintMeta::from_raw(meta);
    let name_fn = not_raw.generate_name_fn();
    let note_fn = not_raw.generate_note_fn();
    let code_fn = not_raw.generate_code_fn();
    let match_with_fn = not_raw.generate_match_with_fn();
    let match_kind = not_raw.generate_match_kind_fn();
    let report_fn = LintMeta::generate_report_fn();

    quote! {
        impl crate::Metadata for #struct_name {
            #name_fn
            #note_fn
            #code_fn
            #match_with_fn
            #match_kind
            #report_fn
        }
    }
}
