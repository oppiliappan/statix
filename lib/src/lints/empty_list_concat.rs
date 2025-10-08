use crate::{Metadata, Report, Rule, Suggestion};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{BinOp, BinOpKind, Expr},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks for concatenations to empty lists
///
/// ## Why is this bad?
/// Concatenation with the empty list is a no-op.
///
/// ## Example
/// ```nix
/// [] ++ something
/// ```
///
/// Remove the operation:
///
/// ```nix
/// something
/// ```
#[lint(
    name = "empty_list_concat",
    note = "Unnecessary concatenation with empty list",
    code = 23,
    match_with = SyntaxKind::NODE_BIN_OP
)]
struct EmptyListConcat;

impl Rule for EmptyListConcat {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let bin_expr = BinOp::cast(node.clone())?;
        let lhs = bin_expr.lhs()?;
        let rhs = bin_expr.rhs()?;
        let Some(BinOpKind::Concat) = bin_expr.operator() else {
            return None;
        };

        let at = node.text_range();
        let message = "Concatenation with the empty list, `[]`, is a no-op";

        let empty_array = if is_empty_array(&lhs) {
            rhs
        } else if is_empty_array(&rhs) {
            lhs
        } else {
            return None;
        };

        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(at, empty_array.syntax().clone()),
        ))
    }
}

fn is_empty_array(expr: &Expr) -> bool {
    let Expr::List(list) = expr else {
        return false;
    };
    list.items().count() == 0
}
