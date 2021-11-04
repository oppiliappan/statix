use crate::{make, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{Apply, Ident, TokenWrapper, TypedNode},
    NodeOrToken, SyntaxElement, SyntaxKind,
};

#[lint(
    name = "deprecated isNull",
    note = "Found usage of deprecated builtin isNull",
    code = 13,
    match_with = SyntaxKind::NODE_APPLY
)]
struct DeprecatedIsNull;

impl Rule for DeprecatedIsNull {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(apply) = Apply::cast(node.clone());
            if let Some(ident) = Ident::cast(apply.lambda()?);
            if ident.as_str() == "isNull";

            if let Some(value) = apply.value();
            then {
                let null = make::ident("null");
                let binop = make::binary(&value, "==", null.node());
                let parenthesized = make::parenthesize(binop.node());

                let at = node.text_range();
                let replacement = parenthesized.node().clone();
                let message = "`isNull` is deprecated, check equality with `null` instead";
                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}
