use crate::{Metadata, Report, Rule, Suggestion, make};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{Attr, AttrpathValue, Expr},
};
use rowan::ast::AstNode as _;

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
    match_with = SyntaxKind::NODE_ATTRPATH_VALUE,
)]
struct ManualInherit;

impl Rule for ManualInherit {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let key_value_stmt = AttrpathValue::cast(node.clone())?;
        let key = key_value_stmt.attrpath()?;
        let mut key_path = key.attrs();
        let key_node = key_path.next()?;

        if key_path.next().is_some() {
            return None;
        }

        let Attr::Ident(key) = key_node else {
            return None;
        };

        let select = key_value_stmt
            .value()
            .and_then(|value| match value {
                // unfortunately we can only match on select although attr path would be enough
                Expr::Select(select) => Some(select),
                _ => None,
            })
            .filter(|select| {
                // hence we have to filter out select statements that have an `or` token in the
                // next step
                select.or_token().is_none()
            })?;

        let select_attrpath = select.attrpath()?;
        let mut select_attrpath_attrs = select_attrpath.attrs();
        let first_attr = select_attrpath_attrs.next()?;

        if select_attrpath_attrs.next().is_some() {
            return None;
        }

        let Attr::Ident(index) = first_attr else {
            return None;
        };

        if key.to_string() != index.to_string() {
            return None;
        }

        let at = node.text_range();

        let replacement = {
            let set = select.expr()?;
            make::inherit_from_stmt(set.syntax(), &[key])
                .syntax()
                .clone()
        };

        Some(self.report().suggest(
            at,
            "This assignment is better written with `inherit`",
            Suggestion::with_replacement(at, replacement),
        ))
    }
}
