use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use sha2::{Digest, Sha256};
use syn::{
    Error, ExprArray, ExprLit, Ident, Lit, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

struct MacroInvocation {
    rule: Ident,
    expressions: Vec<LitStr>,
}

impl Parse for MacroInvocation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const RULE_VALUE: &str = "rule";
        const EXPRESSSIONS_VALUE: &str = "expressions";
        let rule_attribute = input.parse::<Ident>()?;

        if rule_attribute != RULE_VALUE {
            return Err(Error::new(
                rule_attribute.span(),
                "expected `{RULE_VALUE:?}`",
            ));
        }

        input.parse::<Token![:]>()?;
        let rule = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let expressions = input.parse::<Ident>()?;

        if expressions != EXPRESSSIONS_VALUE {
            return Err(Error::new(
                expressions.span(),
                "expected `{EXPRESSSIONS_VALUE:?}`",
            ));
        }

        input.parse::<Token![:]>()?;
        let ExprArray { elems, .. } = input.parse::<ExprArray>()?;

        let expressions = elems
            .into_iter()
            .map(|expr| match expr {
                syn::Expr::Lit(ExprLit {
                    lit: Lit::Str(nix_expression),
                    ..
                }) => Ok(nix_expression),
                _ => Err(Error::new(expr.span(), "expected a literal string")),
            })
            .collect::<Result<Vec<_>, _>>()?;

        input.parse::<Token![,]>()?;
        Ok(MacroInvocation { rule, expressions })
    }
}

pub fn generate_tests(input: TokenStream) -> TokenStream {
    let MacroInvocation { rule, expressions } = parse_macro_input!(input as MacroInvocation);
    expressions
        .into_iter()
        .map(|nix_expression| {
            let lint_test = make_test(&rule, TestKind::Lint, &nix_expression);
            let fix_test = make_test(&rule, TestKind::Fix, &nix_expression);

            quote! {
                #lint_test

                #fix_test
            }
        })
        .collect::<proc_macro2::TokenStream>()
        .into()
}

#[derive(Clone, Copy, Debug)]
enum TestKind {
    Lint,
    Fix,
}

fn make_test(rule: &Ident, kind: TestKind, nix_expression: &LitStr) -> proc_macro2::TokenStream {
    let expression_hash = Sha256::digest(nix_expression.to_token_stream().to_string());
    let expression_hash = hex::encode(expression_hash);

    let kind_str = match kind {
        TestKind::Lint => "lint",
        TestKind::Fix => "fix",
    };

    let test_name = format!("{rule}_{kind_str}_{expression_hash}");
    let test_ident = Ident::new(&test_name, nix_expression.span());
    let snap_name = format!("{kind_str}_{expression_hash}");

    let args = match kind {
        TestKind::Lint => quote! {&["check"]},
        TestKind::Fix => quote! {&["fix", "--dry-run"]},
    };

    quote! {
        #[test]
        fn #test_ident() {
            let expression = #nix_expression;
            let stdout = _utils::test_cli(expression, #args).unwrap();
            insta::assert_snapshot!(#snap_name, stdout, &format!("{expression:?}"));
        }
    }
}
