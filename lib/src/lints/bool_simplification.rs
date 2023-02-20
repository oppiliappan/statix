use crate::{make, session::SessionInfo, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{BinOp, BinOpKind, Paren, TypedNode, UnaryOp, UnaryOpKind, Wrapper},
    NodeOrToken, SyntaxElement, SyntaxKind,
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
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(unary_expr) = UnaryOp::cast(node.clone());
            if unary_expr.operator() == UnaryOpKind::Invert;
            if let Some(value_expr) = unary_expr.value();
            if let Some(paren_expr) = Paren::cast(value_expr);
            if let Some(inner_expr) = paren_expr.inner();
            if let Some(bin_expr) = BinOp::cast(inner_expr);
            if let Some(BinOpKind::Equal) = bin_expr.operator();
            then {
                let at = node.text_range();
                let message = "Try `!=` instead of `!(... == ...)`";

                let lhs = bin_expr.lhs()?;
                let rhs = bin_expr.rhs()?;
                let replacement = make::binary(&lhs, "!=", &rhs).node().clone();
                Some(
                    self.report()
                        .suggest(at, message, Suggestion::new(at, replacement)),
                )
            } else {
                None
            }
        }
    }
}
