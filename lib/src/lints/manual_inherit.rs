use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

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
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let key = KeyValue::cast(node.clone())?.key()?;
        let mut key_path = key.path();
        let key_node = key_path.next()?;

        if key_path.next().is_some() {
            return None;
        }

        let key = Ident::cast(key_node)?;
        let value_node = KeyValue::cast(node.clone())?.value()?;
        let value = Ident::cast(value_node)?;

        if key.as_str() != value.as_str() {
            return None;
        }

        let replacement = make::inherit_stmt(&[key]).node().clone();

        Some(self.report().suggest(
            node.text_range(),
            "This assignment is better written with `inherit`",
            Suggestion::new(node.text_range(), replacement),
        ))
    }
}
