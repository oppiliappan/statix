use crate::{make, Lint, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{KeyValue, Ident, TypedNode, TokenWrapper},
    NodeOrToken, SyntaxElement, SyntaxKind,
};

#[lint(
    name = "manual inherit",
    note = "Assignment instead of inherit",
    code = 3,
    match_with = SyntaxKind::NODE_KEY_VALUE
)]
struct ManualInherit;

impl Rule for ManualInherit {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(key_value_stmt) = KeyValue::cast(node.clone());
            if let Some(key_path) = key_value_stmt.key();
            if let Some(key_node) = key_path.path().next();
            if let Some(key) = Ident::cast(key_node);

            if let Some(value_node) = key_value_stmt.value();
            if let Some(value) = Ident::cast(value_node);

            if key.as_str() == value.as_str();

            then {
                let at = node.text_range();
                let replacement = make::inherit_stmt(&[key]).node().clone();
                let message = format!("The assignment `{}` is better written with `inherit`", node);
                Some(Self::report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}

