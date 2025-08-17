use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{AttrpathValue, Ident},
};
use rowan::ast::AstNode;

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
        if let NodeOrToken::Node(node) = node {
            let key_value_stmt = AttrpathValue::cast(node.clone())?;
            let key_path = key_value_stmt.attrpath()?;
            let mut key_path_iter = key_path.attrs();
            let key_node = key_path_iter.next()?;
            // ensure that path has exactly one component
            key_path_iter.next().is_none().then_some(())?;
            let key = Ident::cast(key_node.syntax().clone())?;

            let value_node = key_value_stmt.value()?;
            let value = Ident::cast(value_node.syntax().clone())?;

            (key.to_string() == value.to_string()).then_some(())?;
            let at = node.text_range();
            let replacement = make::inherit_stmt(&[key]);
            let message = "This assignment is better written with `inherit`";
            Some(self.report().suggest(
                at,
                message,
                Suggestion::new(at, Some(replacement.syntax().clone())),
            ))
        } else {
            None
        }
    }
}
