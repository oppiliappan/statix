use crate::{make, session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{
    ast::{AttrpathValue, Ident, Select},
    NodeOrToken, SyntaxElement, SyntaxKind,
};

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
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(key_value_stmt) = AttrpathValue::cast(node.clone());
            if let mut key_path = key_value_stmt.attrpath()?.attrs();
            if let Some(key_node) = key_path.next();
            // ensure that path has exactly one component
            if key_path.next().is_none();
            if let Some(key) = Ident::cast(key_node.syntax().clone());

            if let Some(value_node) = key_value_stmt.value();
            if let Some(value) = Select::cast(value_node.syntax().clone());
            if let Some(index_node) = value.expr();
            if let Some(index) = Ident::cast(index_node.syntax().clone());

            if key.to_string() == index.to_string();

            then {
                let at = node.text_range();
                let replacement = {
                    let set = value.attrpath()?;
                    make::inherit_from_stmt(set.syntax().clone(), &[key]).syntax().clone()
                };
                let message = "This assignment is better written with `inherit`";
                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}
