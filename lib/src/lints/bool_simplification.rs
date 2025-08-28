use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    types::{BinOp, BinOpKind, Paren, TypedNode, UnaryOp, UnaryOpKind, Wrapper},
};

/// ## What it does
/// Checks for boolean expressions that can be simplified.
///
/// ## Why is this bad?
/// Complex booleans affect readibility.
///
/// ## Example
/// ```nix
/// if !(x == y) then 0 else 1
/// ```
///
/// Use `!=` instead:
///
/// ```nix
/// if x != y then 0 else 1
/// ```
#[lint(
    name = "bool_simplification",
    note = "This boolean expression can be simplified",
    code = 18,
    match_with = SyntaxKind::NODE_UNARY_OP
)]
struct BoolSimplification;

impl Rule for BoolSimplification {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let unary_expr = UnaryOp::cast(node.clone())?;

        if unary_expr.operator() != UnaryOpKind::Invert {
            return None;
        }

        let value_expr = unary_expr.value()?;
        let paren_expr = Paren::cast(value_expr)?;
        let inner_expr = paren_expr.inner()?;
        let bin_expr = BinOp::cast(inner_expr)?;

        let Some(BinOpKind::Equal) = bin_expr.operator() else {
            return None;
        };

        let at = node.text_range();
        let message = "Try `!=` instead of `!(... == ...)`";

        let lhs = bin_expr.lhs()?;
        let rhs = bin_expr.rhs()?;
        let replacement = make::binary(&lhs, "!=", &rhs).node().clone();
        Some(
            self.report()
                .suggest(at, message, Suggestion::with_replacement(at, replacement)),
        )
    }
}
