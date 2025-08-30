use crate::{Diagnostic, Metadata, Report, Rule, Suggestion, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{AttrpathValue, Expr, LetIn, Paren},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks for unnecessary parentheses.
///
/// ## Why is this bad?
/// Unnecessarily parenthesized code is hard to read.
///
/// ## Example
///
/// ```nix
/// let
///   double = (x: 2 * x);
///   ls = map (double) [ 1 2 3 ];
/// in
///   (2 + 3)
/// ```
///
/// Remove unnecessary parentheses:
///
/// ```nix
/// let
///   double = x: 2 * x;
///   ls = map double [ 1 2 3 ];
/// in
///   2 + 3
/// ```
#[lint(
    name = "useless_parens",
    note = "These parentheses can be omitted",
    code = 8,
    match_with = [
        SyntaxKind::NODE_ATTRPATH_VALUE,
        SyntaxKind::NODE_PAREN,
        SyntaxKind::NODE_LET_IN,
    ]
)]
struct UselessParens;

impl Rule for UselessParens {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let diagnostic = match (AttrpathValue::cast(node.clone()), Expr::cast(node.clone())) {
            (Some(attrpath_value), _) => {
                let value_node = attrpath_value.value()?;
                let value_range = value_node.syntax().text_range();
                let paren = Paren::cast(value_node.syntax().clone())?;
                let suggestion =
                    Suggestion::with_replacement(value_range, paren.expr()?.syntax().clone());

                Diagnostic::suggest(
                    value_range,
                    "Useless parentheses around value in binding",
                    suggestion,
                )
            }
            (_, Some(Expr::LetIn(let_in))) => {
                let body_node = let_in.body()?;
                let body_range = body_node.syntax().text_range();
                let paren = Paren::cast(body_node.syntax().clone())?;
                let suggestion =
                    Suggestion::with_replacement(body_range, paren.expr()?.syntax().clone());

                Diagnostic::suggest(
                    body_range,
                    "Useless parentheses around body of `let` expression",
                    suggestion,
                )
            }
            (_, Some(Expr::Paren(paren_expr))) => {
                let paren_expr_range = paren_expr.syntax().text_range();
                let father_node = paren_expr.syntax().parent()?;

                // ensure that we don't lint inside let-in statements
                // we already lint such cases in previous match stmt
                if AttrpathValue::cast(father_node.clone()).is_some() {
                    return None;
                }

                // ensure that we don't lint inside let-bodies
                // if this primitive is a let-body, we have already linted it
                if LetIn::cast(father_node).is_some() {
                    return None;
                }

                let parsed_inner = Expr::cast(paren_expr.expr()?.syntax().clone())?;

                match &parsed_inner {
                    Expr::List(_)
                    | Expr::Paren(_)
                    | Expr::Str(_)
                    | Expr::AttrSet(_)
                    | Expr::Ident(_) => {}
                    Expr::Select(select) if select.or_token().is_none() => {}
                    _ => return None,
                }

                Diagnostic::suggest(
                    paren_expr_range,
                    "Useless parentheses around primitive expression",
                    Suggestion::with_replacement(paren_expr_range, parsed_inner.syntax().clone()),
                )
            }
            _ => return None,
        };

        let mut report = self.report();
        report.diagnostics.push(diagnostic);
        Some(report)
    }
}
