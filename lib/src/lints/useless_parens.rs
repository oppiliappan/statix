use crate::{Diagnostic, Metadata, Report, Rule, Suggestion, session::SessionInfo};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{AttrpathValue, Entry, Expr, LetIn, Paren},
};

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
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(diagnostic) = do_thing(node);
            then {
                let mut report = self.report();
                report.diagnostics.push(diagnostic);
                Some(report)
            } else {
                None
            }
        }
    }
}

fn do_thing(node: &SyntaxNode) -> Option<Diagnostic> {
    match (Entry::cast(node.clone()), Expr::cast(node.clone())) {
        (Some(Entry::AttrpathValue(attrpath_value)), _) => if_chain! {
            if let Some(value_node) = attrpath_value.value();
            let value_range = value_node.syntax().text_range();
            if let Some(value_in_parens) = Paren::cast(value_node.syntax().clone());
            if let Some(inner) = value_in_parens.expr();
            then {
                let at = value_range;
                let message = "Useless parentheses around value in binding";
                let replacement = inner;
                Some(Diagnostic::suggest(at, message, Suggestion::new(at, replacement.syntax().clone())))
            } else {
                None
            }
        },
        (_, Some(Expr::LetIn(let_in))) => if_chain! {
            if let Some(body_node) = let_in.body();
            let body_range = body_node.syntax().text_range();
            if let Some(body_as_parens) = Paren::cast(body_node.syntax().clone());
            if let Some(inner) = body_as_parens.expr();
            then {
                let at = body_range;
                let message = "Useless parentheses around body of `let` expression";
                let replacement = inner;
                Some(Diagnostic::suggest(at, message, Suggestion::new(at, replacement.syntax().clone())))
            } else {
                None
            }
        },
        (_, Some(Expr::Paren(paren_expr))) => if_chain! {
            let paren_expr_range = paren_expr.syntax().text_range();
            if let Some(father_node) = paren_expr.syntax().parent();

            // ensure that we don't lint inside let-in statements
            // we already lint such cases in previous match stmt
            if AttrpathValue::cast(father_node.clone()).is_none();

            // ensure that we don't lint inside let-bodies
            // if this primitive is a let-body, we have already linted it
            if LetIn::cast(father_node).is_none();

            if let Some(inner_node) = paren_expr.expr();
            if let Some(parsed_inner) = Expr::cast(inner_node.syntax().clone());
            if matches!(
                parsed_inner,
                Expr::List(_)
                    | Expr::Paren(_)
                    | Expr::Str(_)
                    | Expr::AttrSet(_)
                    | Expr::Select(_)
                    | Expr::Ident(_)
            );
            then {
                let at = paren_expr_range;
                let message = "Useless parentheses around primitive expression";
                let replacement = parsed_inner;
                Some(Diagnostic::suggest(at, message, Suggestion::new(at, replacement.syntax().clone())))
            } else {
                None
            }
        },
        _ => None,
    }
}
