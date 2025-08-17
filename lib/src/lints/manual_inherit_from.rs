use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{AttrpathValue, Ident, Select},
};
use rowan::ast::AstNode;

/// ## What it does
/// Checks for bindings of the form `a = someAttr.a`.
///
/// ## Why is this bad?
/// If the aim is to extract or bring attributes of an attrset into
/// scope, prefer an inherit statement.
///
/// ## Example
///
/// ```nix
/// let
///   mtl = pkgs.haskellPackages.mtl;
/// in
///   null
/// ```
///
/// Try `inherit` instead:
///
/// ```nix
/// let
///   inherit (pkgs.haskellPackages) mtl;
/// in
///   null
/// ```
#[lint(
    name = "manual_inherit_from",
    note = "Assignment instead of inherit from",
    code = 4,
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
            let value = Select::cast(value_node.syntax().clone())?;
            let index_node = value.attrpath().and_then(|attrs| attrs.attrs().next())?;
            let index = Ident::cast(index_node.syntax().clone())?;

            (key.to_string() == index.to_string()).then_some(())?;

            let at = node.text_range();
            let replacement = {
                let set = value.expr()?;
                make::inherit_from_stmt(set.syntax(), &[key])
            };
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
