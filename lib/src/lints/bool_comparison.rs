use crate::{make, session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use macros::lint;
use rnix::{
    ast::{BinOp, BinOpKind, Ident},
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
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
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        let bin_expr = BinOp::cast(node.clone())?;
        let lhs = bin_expr.lhs()?;
        let rhs = bin_expr.rhs()?;
        let op = bin_expr.operator()?;

        let (BinOpKind::Equal | BinOpKind::NotEqual) = op else {
            return None;
        };

        let (non_bool_side, bool_side) = boolean_ident(lhs.syntax())
            .map(|bool_side| (&rhs, bool_side))
            .or_else(|| boolean_ident(rhs.syntax()).map(|bool_side| (&lhs, bool_side)))?;

        let at = node.text_range();
        let replacement = {
            match (bool_side, op == BinOpKind::Equal) {
                (NixBoolean::True, true) | (NixBoolean::False, false) => {
                    // `a == true`, `a != false` replace with just `a`
                    non_bool_side.clone()
                }
                (NixBoolean::True, false) | (NixBoolean::False, true) => {
                    // `a != true`, `a == false` replace with `!a`
                    let unary_op = match non_bool_side.syntax().kind() {
                        SyntaxKind::NODE_APPLY
                        | SyntaxKind::NODE_PAREN
                        | SyntaxKind::NODE_IDENT => {
                            // do not parenthsize the replacement
                            make::unary_not(non_bool_side.syntax())
                        }
                        SyntaxKind::NODE_BIN_OP => {
                            let inner = BinOp::cast(non_bool_side.syntax().clone()).unwrap();
                            // `!a ? b`, no paren required
                            if inner.operator()? == BinOpKind::Or {
                                make::unary_not(non_bool_side.syntax())
                            } else {
                                let parens = make::parenthesize(non_bool_side.syntax());
                                make::unary_not(parens.syntax())
                            }
                        }
                        _ => {
                            let parens = make::parenthesize(non_bool_side.syntax());
                            make::unary_not(parens.syntax())
                        }
                    };
                    rnix::ast::Expr::UnaryOp(unary_op)
                }
            }
        };
        let message = format!("Comparing `{non_bool_side}` with boolean literal `{bool_side}`",);
        Some(self.report().suggest(
            at,
            message,
            Suggestion::new(at, replacement.syntax().clone()),
        ))
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
