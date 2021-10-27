use crate::{make, Lint, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{Dynamic, TypedNode},
    NodeOrToken, SyntaxElement, SyntaxKind,
};

#[lint(
    name = "unquoted splice",
    note = "Found unquoted splice expression",
    code = 9,
    match_with = SyntaxKind::NODE_DYNAMIC
)]
struct UnquotedSplice;

impl Rule for UnquotedSplice {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if Dynamic::cast(node.clone()).is_some();
            then {
                let at = node.text_range();
                let replacement = make::quote(&node).node().clone();
                let message = "Consider quoting this splice expression";
                Some(Self::report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}
