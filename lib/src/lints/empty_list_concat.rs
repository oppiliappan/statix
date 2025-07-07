use crate::{Metadata, Report, Rule, Suggestion, session::SessionInfo};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{BinOp, BinOpKind, List},
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
                if is_empty_array(&lhs.syntax()) {
                    Some(self.report().suggest(at, message, Suggestion::new(at, rhs.syntax().clone())))
                } else if is_empty_array(&rhs.syntax()) {
                    Some(self.report().suggest(at, message, Suggestion::new(at, lhs.syntax().clone())))
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
    List::cast(node.clone())
        .map(|list| list.items().count() == 0)
        .unwrap_or_default()
}
