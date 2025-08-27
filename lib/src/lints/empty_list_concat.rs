use crate::{Metadata, Report, Rule, Suggestion, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    types::{BinOp, BinOpKind, List, TypedNode},
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
        let Some(BinOpKind::Concat) = bin_expr.operator() else {
            return None;
        };

        let at = node.text_range();
        let message = "Concatenation with the empty list, `[]`, is a no-op";
        if is_empty_array(&lhs) {
            Some(self.report().suggest(at, message, Suggestion::new(at, rhs)))
        } else if is_empty_array(&rhs) {
            Some(self.report().suggest(at, message, Suggestion::new(at, lhs)))
        } else {
            None
        }
    }
}

fn is_empty_array(node: &SyntaxNode) -> bool {
    List::cast(node.clone()).is_some_and(|list| list.items().count() == 0)
}
