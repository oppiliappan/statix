use crate::{make, Lint, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{KeyValue, Ident, Select, TypedNode, TokenWrapper},
    NodeOrToken, SyntaxElement, SyntaxKind,
};

#[lint(
    name = "manual inherit from",
    note = "Assignment instead of inherit from",
    code = 4,
    match_with = SyntaxKind::NODE_KEY_VALUE
)]
struct ManualInherit;

impl Rule for ManualInherit {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(key_value_node) = node;
            if let Some(key_value_stmt) = KeyValue::cast(key_value_node.clone());
            if let Some(key_path) = key_value_stmt.key();
            if let Some(key_node) = key_path.path().next();
            if let Some(key) = Ident::cast(key_node);

            if let Some(value_node) = key_value_stmt.value();
            if let Some(value) = Select::cast(value_node);
            if let Some(index_node) = value.index();
            if let Some(index) = Ident::cast(index_node);

            if key.as_str() == index.as_str();

            then {
                let at = node.text_range();
                let replacement = {
                    let set = value.set()?;
                    make::inherit_from_stmt(set, &[key]).node().clone() 
                };
                let message = format!("The assignment `{}` is better written with `inherit`", node);
                Some(Self::report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}


