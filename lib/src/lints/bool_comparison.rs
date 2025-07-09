use crate::{make, session::SessionInfo, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{BinOp, BinOpKind, Ident, TokenWrapper, TypedNode},
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
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(bin_expr) = BinOp::cast(node.clone());
            if let Some(lhs) = bin_expr.lhs();
            if let Some(rhs) = bin_expr.rhs();
            if let Some(op) = bin_expr.operator();

            if let BinOpKind::Equal | BinOpKind::NotEqual = op;
            let (non_bool_side, bool_side) = if boolean_ident(&lhs).is_some() {
                (rhs, lhs)
            } else if boolean_ident(&rhs).is_some() {
                (lhs, rhs)
            } else {
                return None
            };
            then {
                let at = node.text_range();
                let replacement = {
                    match (boolean_ident(&bool_side).unwrap(), op == BinOpKind::Equal) {
                        (NixBoolean::True, true) | (NixBoolean::False, false) => {
                            // `a == true`, `a != false` replace with just `a`
                            non_bool_side.clone()
                        },
                        (NixBoolean::True, false) | (NixBoolean::False, true) => {
                            // `a != true`, `a == false` replace with `!a`
                            match non_bool_side.kind() {
                                SyntaxKind::NODE_APPLY
                                    | SyntaxKind::NODE_PAREN
                                    | SyntaxKind::NODE_IDENT => {
                                    // do not parenthsize the replacement
                                    make::unary_not(&non_bool_side).node().clone()
                                },
                                SyntaxKind::NODE_BIN_OP => {
                                    let inner = BinOp::cast(non_bool_side.clone()).unwrap();
                                    // `!a ? b`, no paren required
                                    if inner.operator()? == BinOpKind::IsSet {
                                        make::unary_not(&non_bool_side).node().clone()
                                    } else {
                                        let parens = make::parenthesize(&non_bool_side);
                                        make::unary_not(parens.node()).node().clone()
                                    }
                                },
                                _ => {
                                    let parens = make::parenthesize(&non_bool_side);
                                    make::unary_not(parens.node()).node().clone()
                                }
                            }
                        },
                    }
                };
                let message = format!(
                    "Comparing `{}` with boolean literal `{}`",
                    non_bool_side,
                    bool_side
                );
                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}

enum NixBoolean {
    True,
    False,
}

// not entirely accurate, underhanded nix programmers might write `true = false`
fn boolean_ident(node: &SyntaxNode) -> Option<NixBoolean> {
    Ident::cast(node.clone()).and_then(|ident_expr| match ident_expr.as_str() {
        "true" => Some(NixBoolean::True),
        "false" => Some(NixBoolean::False),
        _ => None,
    })
}
