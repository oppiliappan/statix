use crate::{session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use macros::lint;
use rnix::{
    ast::{BinOp, BinOpKind, List},
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
};

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
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        let bin_expr = BinOp::cast(node.clone())?;
        let lhs = bin_expr.lhs()?;
        let rhs = bin_expr.rhs()?;
        let op = bin_expr.operator()?;
        let BinOpKind::Concat = op else {
            return None;
        };

        let at = node.text_range();
        let message = "Concatenation with the empty list, `[]`, is a no-op";
        is_empty_array(lhs.syntax())
            .then(|| {
                self.report()
                    .suggest(at, message, Suggestion::new(at, rhs.syntax().clone()))
            })
            .or_else(|| {
                is_empty_array(rhs.syntax()).then(|| {
                    self.report()
                        .suggest(at, message, Suggestion::new(at, lhs.syntax().clone()))
                })
            })
    }
}

fn is_empty_array(node: &SyntaxNode) -> bool {
    List::cast(node.clone())
        .map(|list| list.items().count() == 0)
        .unwrap_or_default()
}
