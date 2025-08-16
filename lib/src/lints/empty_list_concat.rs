use crate::{session::SessionInfo, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{BinOp, BinOpKind, List, TypedNode},
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
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(bin_expr) = BinOp::cast(node.clone());
            if let Some(lhs) = bin_expr.lhs();
            if let Some(rhs) = bin_expr.rhs();
            if let Some(op) = bin_expr.operator();
            if let BinOpKind::Concat = op;
            then {
                let at = node.text_range();
                let message = "Concatenation with the empty list, `[]`, is a no-op";
                if is_empty_array(&lhs) {
                    Some(self.report().suggest(at, message, Suggestion::new(at, rhs)))
                } else if is_empty_array(&rhs) {
                    Some(self.report().suggest(at, message, Suggestion::new(at, lhs)))
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

fn is_empty_array(node: &SyntaxNode) -> bool {
    List::cast(node.clone()).is_some_and(|list| list.items().count() == 0)
}
