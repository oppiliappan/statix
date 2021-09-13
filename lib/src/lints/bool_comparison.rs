use crate::{Diagnostic, Lint, Metadata, Rule};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{BinOp, BinOpKind, Ident, TokenWrapper, TypedNode},
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
};

#[lint(
    name = "bool_comparison",
    note = "Unnecessary comparison with boolean",
    match_with = "SyntaxKind::NODE_BIN_OP"
)]
struct BoolComparison;

impl Rule for BoolComparison {
    fn validate(&self, node: &SyntaxElement) -> Option<Diagnostic> {
        if_chain! {
            if let NodeOrToken::Node(bin_op_node) = node;
            if let Some(bin_expr) = BinOp::cast(bin_op_node.clone());
            if let Some(lhs) = bin_expr.lhs();
            if let Some(rhs) = bin_expr.rhs();

            if let BinOpKind::Equal | BinOpKind::NotEqual = bin_expr.operator();
            let (non_bool_side, bool_side) = if is_boolean_ident(&lhs) {
                (rhs, lhs)
            } else if is_boolean_ident(&rhs) {
                (lhs, rhs)
            } else {
                return None
            };
            then {
                let at = node.text_range();
                let message = format!(
                    "Comparing `{}` with boolean literal `{}`",
                    non_bool_side,
                    bool_side
                );
                dbg!(Some(Diagnostic::new(at, message)))
            } else {
                None
            }
        }
    }
}

// not entirely accurate, underhanded nix programmers might write `true = false`
fn is_boolean_ident(node: &SyntaxNode) -> bool {
    if let Some(ident_expr) = Ident::cast(node.clone()) {
        ident_expr.as_str() == "true" || ident_expr.as_str() == "false"
    } else {
        false
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use rnix::{parser, WalkEvent};
//
//     #[test]
//     fn trivial() {
//         let src = r#"
//         a == true
//         "#;
//         let parsed = rnix::parse(src).as_result().ok().unwrap();
//         let _ = parsed
//             .node()
//             .preorder_with_tokens()
//             .filter_map(|event| match event {
//                 WalkEvent::Enter(t) => Some(t),
//                 _ => None,
//             })
//             .map(|node| BoolComparison.validate(&node))
//             .collect::<Vec<_>>();
//     }
// }
