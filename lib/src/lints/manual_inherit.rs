use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{Attr, AttrpathValue, Expr},
};
use rowan::ast::AstNode as _;

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
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let attrpath_value = AttrpathValue::cast(node.clone())?;
        let attrpath = attrpath_value.attrpath()?;
        let mut attrs = attrpath.attrs();
        let first_attr = attrs.next()?;

        if attrs.next().is_some() {
            return None;
        }

        let Attr::Ident(key) = first_attr else {
            return None;
        };

        let Some(Expr::Ident(value)) = attrpath_value.value() else {
            return None;
        };

        if key.to_string() != value.to_string() {
            return None;
        }

        let replacement = make::inherit_stmt(&[key]).syntax().clone();

        Some(self.report().suggest(
            node.text_range(),
            "This assignment is better written with `inherit`",
            Suggestion::with_replacement(node.text_range(), replacement),
        ))
    }
}
