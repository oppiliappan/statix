use crate::{make, session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{
    ast::{AttrpathValue, Ident},
    NodeOrToken, SyntaxElement, SyntaxKind,
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
    match_with = SyntaxKind::NODE_ATTRPATH_VALUE
)]
struct ManualInherit;

impl Rule for ManualInherit {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(key_value_stmt) = AttrpathValue::cast(node.clone());
            if let key_path = key_value_stmt.attrpath()?;
            if let Some(key_node) = key_path.attrs().next();
            // ensure that path has exactly one component
            if key_path.attrs().count() == 1;
            if let Some(key) = Ident::cast(key_node.syntax().clone());

            if let Some(value_node) = key_value_stmt.value();
            if let Some(value) = Ident::cast(value_node.syntax().clone());

            if key.to_string() == value.to_string();

            then {
                let at = node.text_range();
                let replacement = make::inherit_stmt(&[key]).syntax().clone();
                let message = "This assignment is better written with `inherit`";
                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}
