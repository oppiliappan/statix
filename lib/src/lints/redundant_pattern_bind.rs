use crate::{Lint, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{Pattern, TokenWrapper, TypedNode},
    NodeOrToken, SyntaxElement, SyntaxKind,
};

#[lint(
    name = "redundant pattern bind",
    note = "Found redundant pattern bind in function argument",
    code = 10,
    match_with = SyntaxKind::NODE_PATTERN
)]
struct RedundantPatternBind;

impl Rule for RedundantPatternBind {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(pattern) = Pattern::cast(node.clone());
            // no patterns within `{ }`
            if pattern.entries().count() == 0;

            // pattern is just ellipsis
            if pattern.ellipsis();

            // pattern is bound
            if let Some(ident) =  pattern.at();
            then {
                let at = node.text_range();
                let message = format!("This pattern bind is redundant, use `{}` instead", ident.as_str());
                let replacement = ident.node().clone();
                Some(Self::report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}
