use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    types::{Ident, KeyValue, Select, TokenWrapper, TypedNode},
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
    match_with = SyntaxKind::NODE_KEY_VALUE
)]
struct ManualInherit;

impl Rule for ManualInherit {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if let NodeOrToken::Node(node) = node
            && let Some(key_value_stmt) = KeyValue::cast(node.clone())
            && let mut key_path = key_value_stmt.key()?.path()
            && let Some(key_node) = key_path.next()
            // ensure that path has exactly one component
            && key_path.next().is_none()
            && let Some(key) = Ident::cast(key_node)
            && let Some(value_node) = key_value_stmt.value()
            && let Some(value) = Select::cast(value_node)
            && let Some(index_node) = value.index()
            && let Some(index) = Ident::cast(index_node)
            && key.as_str() == index.as_str()
        {
            let at = node.text_range();
            let replacement = {
                let set = value.set()?;
                make::inherit_from_stmt(set, &[key]).node().clone()
            };
            let message = "This assignment is better written with `inherit`";
            Some(
                self.report()
                    .suggest(at, message, Suggestion::new(at, replacement)),
            )
        } else {
            None
        }
    }
}
