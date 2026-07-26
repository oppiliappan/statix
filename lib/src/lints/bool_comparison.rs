use crate::{Metadata, Report, Rule, Suggestion, make};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{BinOp, BinOpKind, Expr, Ident},
};
use rowan::ast::AstNode as _;

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
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        let bin_expr = BinOp::cast(node.clone())?;
        let (lhs, rhs) = (bin_expr.lhs()?, bin_expr.rhs()?);
        if [&lhs, &rhs]
            .iter()
            .any(|expr| !matches!(expr, Expr::Literal(_) | Expr::Ident(_) | Expr::HasAttr(_)))
        {
            return None;
        }
        let (lhs, rhs) = (lhs.syntax(), rhs.syntax());
        let op = EqualityBinOpKind::try_from(bin_expr.operator()?)?;

        let (bool_side, non_bool_side): (NixBoolean, &SyntaxNode) =
            match (boolean_ident(lhs), boolean_ident(rhs)) {
                (None, None) => return None,
                (None, Some(bool)) => (bool, lhs),
                (Some(bool), _) => (bool, rhs),
            };

        let replacement = match (&bool_side, op) {
            (NixBoolean::True, EqualityBinOpKind::Equal)
            | (NixBoolean::False, EqualityBinOpKind::NotEqual) => {
                // `a == true`, `a != false` replace with just `a`
                non_bool_side.clone()
            }
            (NixBoolean::True, EqualityBinOpKind::NotEqual)
            | (NixBoolean::False, EqualityBinOpKind::Equal) => {
                // `a != true`, `a == false` replace with `!a`
                match non_bool_side.kind() {
                    SyntaxKind::NODE_APPLY
                    | SyntaxKind::NODE_PAREN
                    | SyntaxKind::NODE_IDENT
                    | SyntaxKind::NODE_HAS_ATTR => {
                        // do not parenthsize the replacement
                        make::unary_not(non_bool_side).syntax().clone()
                    }
                    _ => {
                        let parens = make::parenthesize(non_bool_side);
                        make::unary_not(parens.syntax()).syntax().clone()
                    }
                }
            }
        };
        let at = node.text_range();
        Some(self.report().suggest(
            at,
            format!("Comparing '{non_bool_side}' with boolean literal '{bool_side}'. We can't check the type of '{non_bool_side}'. Please verify that holds a boolean expression."),
            Suggestion::with_replacement(at, replacement),
        ))
    }
}

enum NixBoolean {
    True,
    False,
}

#[derive(Debug)]
enum EqualityBinOpKind {
    NotEqual,
    Equal,
}

impl EqualityBinOpKind {
    /// Try to create from a `BinOpKind`
    ///
    /// Returns an option, not a Result
    fn try_from(bin_op_kind: BinOpKind) -> Option<Self> {
        match bin_op_kind {
            BinOpKind::Equal => Some(Self::Equal),
            BinOpKind::NotEqual => Some(Self::NotEqual),
            _ => None,
        }
    }
}

impl std::fmt::Display for NixBoolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::True => "true",
            Self::False => "false",
        };
        write!(f, "{s}")
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
