use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{BinOp, BinOpKind, Ident},
};

/// ## What it does
/// Checks for expressions of the form `x == true`, `x != true` and
/// suggests using the variable directly.
///
/// ## Why is this bad?
/// Unnecessary code.
///
/// ## Example
/// Instead of checking the value of `x`:
///
/// ```nix
/// if x == true then 0 else 1
/// ```
///
/// Use `x` directly:
///
/// ```nix
/// if x then 0 else 1
/// ```
#[lint(
    name = "bool_comparison",
    note = "Unnecessary comparison with boolean",
    code = 1,
    match_with = SyntaxKind::NODE_BIN_OP
)]
struct BoolComparison;

impl Rule for BoolComparison {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(bin_expr) = BinOp::cast(node.clone());
            if let Some(lhs) = bin_expr.lhs();
            if let Some(rhs) = bin_expr.rhs();
            if let Some(op) = bin_expr.operator();

            if let BinOpKind::Equal | BinOpKind::NotEqual = op;
            let (non_bool_side, bool_side) = if boolean_ident(lhs.syntax()).is_some() {
                (rhs, lhs)
            } else if boolean_ident(rhs.syntax()).is_some() {
                (lhs, rhs)
            } else {
                return None
            };
            then {
                let at = node.text_range();
                let replacement = {
                    match (boolean_ident(bool_side.syntax()).unwrap(), op == BinOpKind::Equal) {
                        (NixBoolean::True, true) | (NixBoolean::False, false) => {
                            // `a == true`, `a != false` replace with just `a`
                            non_bool_side.clone()
                        },
                        (NixBoolean::True, false) | (NixBoolean::False, true) => {
                            // `a != true`, `a == false` replace with `!a`
                            let unary_op = match non_bool_side.syntax().kind() {
                                SyntaxKind::NODE_APPLY
                                    | SyntaxKind::NODE_PAREN
                                    | SyntaxKind::NODE_HAS_ATTR
                                    | SyntaxKind::NODE_IDENT => {
                                    // do not parenthsize the replacement
                                    make::unary_not(&non_bool_side.syntax())
                                },
                                SyntaxKind::NODE_BIN_OP => {
                                    let parens = make::parenthesize(non_bool_side.syntax());
                                    make::unary_not(parens.syntax())
                                },
                                _ => {
                                    let parens = make::parenthesize(non_bool_side.syntax());
                                    make::unary_not(parens.syntax())
                                }
                            };
                            rnix::ast::Expr::UnaryOp(unary_op)
                        },
                    }
                };
                let message = format!(
                    "Comparing `{}` with boolean literal `{}`",
                    non_bool_side,
                    bool_side
                );
                Some(self.report().suggest(at, message, Suggestion::new(at, replacement.syntax().clone())))
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Copy)]
enum NixBoolean {
    True,
    False,
}

impl std::fmt::Display for NixBoolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if matches!(self, Self::True) {
                "true"
            } else {
                "false"
            }
        )
    }
}

// not entirely accurate, underhanded nix programmers might write `true = false`
fn boolean_ident(node: &SyntaxNode) -> Option<NixBoolean> {
    Ident::cast(node.clone()).and_then(|ident_expr| match ident_expr.to_string().as_str() {
        "true" => Some(NixBoolean::True),
        "false" => Some(NixBoolean::False),
        _ => None,
    })
}
