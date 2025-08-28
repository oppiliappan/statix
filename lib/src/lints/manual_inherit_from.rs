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
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let key_value_stmt = KeyValue::cast(node.clone())?;
        let key = key_value_stmt.key()?;
        let mut key_path = key.path();
        let key_node = key_path.next()?;

        if key_path.next().is_some() {
            return None;
        }

        let key = Ident::cast(key_node)?;
        let value = Select::cast(key_value_stmt.value()?)?;
        let index = Ident::cast(value.index()?)?;

        if key.as_str() != index.as_str() {
            return None;
        }

        let at = node.text_range();
        let replacement = {
            let set = value.set()?;
            make::inherit_from_stmt(&set, &[key]).node().clone()
        };

        Some(self.report().suggest(
            at,
            "This assignment is better written with `inherit`",
            Suggestion::with_replacement(at, replacement),
        ))
    }
}
