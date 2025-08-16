use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    types::{Ident, KeyValue, TokenWrapper, TypedNode},
};

/// ## What it does
/// Checks for bindings of the form `a = a`.
///
/// ## Why is this bad?
/// If the aim is to bring attributes from a larger scope into
/// the current scope, prefer an inherit statement.
///
/// ## Example
///
/// ```nix
/// let
///   a = 2;
/// in
///   { a = a; b = 3; }
/// ```
///
/// Try `inherit` instead:
///
/// ```nix
/// let
///   a = 2;
/// in
///   { inherit a; b = 3; }
/// ```
#[lint(
    name = "manual_inherit",
    note = "Assignment instead of inherit",
    code = 3,
    match_with = SyntaxKind::NODE_KEY_VALUE
)]
struct ManualInherit;

impl Rule for ManualInherit {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(key_value_stmt) = KeyValue::cast(node.clone());
            if let mut key_path = key_value_stmt.key()?.path();
            if let Some(key_node) = key_path.next();
            // ensure that path has exactly one component
            if key_path.next().is_none();
            if let Some(key) = Ident::cast(key_node);

            if let Some(value_node) = key_value_stmt.value();
            if let Some(value) = Ident::cast(value_node);

            if key.as_str() == value.as_str();

            then {
                let at = node.text_range();
                let replacement = make::inherit_stmt(&[key]).node().clone();
                let message = "This assignment is better written with `inherit`";
                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}
