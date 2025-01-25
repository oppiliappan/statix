use crate::{session::SessionInfo, Diagnostic, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use macros::lint;
use rnix::{
    ast::{AttrpathValue, Entry, Expr, LetIn, Paren},
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
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
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        check_node(node).map(|diagnostic| {
            let mut report = self.report();
            report.diagnostics.push(diagnostic);
            report
        })
    }
}

fn check_node(node: &SyntaxNode) -> Option<Diagnostic> {
    Entry::cast(node.clone())
        .and_then(|entry| match entry {
            Entry::AttrpathValue(attrpath_value) => check_attrpathvalue(attrpath_value),
            _ => None,
        })
        .or_else(|| {
            Expr::cast(node.clone()).and_then(|expr| match expr {
                Expr::LetIn(let_in) => check_let_in(let_in),
                Expr::Paren(paren) => check_paren(paren),
                _ => None,
            })
        })
}

fn check_attrpathvalue(attrpath_value: AttrpathValue) -> Option<Diagnostic> {
    let value_node = attrpath_value.value()?;
    let value_range = value_node.syntax().text_range();
    let value_in_parens = Paren::cast(value_node.syntax().clone())?;
    let inner = value_in_parens.expr()?;

    let at = value_range;
    let message = "Useless parentheses around value in binding";
    let replacement = inner;
    Some(Diagnostic::suggest(
        at,
        message,
        Suggestion::new(at, replacement.syntax().clone()),
    ))
}

fn check_let_in(let_in: LetIn) -> Option<Diagnostic> {
    let body_node = let_in.body()?;
    let body_range = body_node.syntax().text_range();
    let body_as_parens = Paren::cast(body_node.syntax().clone())?;
    let inner = body_as_parens.expr()?;

    let at = body_range;
    let message = "Useless parentheses around body of `let` expression";
    let replacement = inner;
    Some(Diagnostic::suggest(
        at,
        message,
        Suggestion::new(at, replacement.syntax().clone()),
    ))
}

fn check_paren(paren: Paren) -> Option<Diagnostic> {
    let paren_expr_range = paren.syntax().text_range();
    let parent_node = paren.syntax().parent()?;

    // ensure that we don't lint inside let-in statements
    // we already lint such cases in previous match stmt
    if AttrpathValue::cast(parent_node.clone()).is_some() {
        return None;
    };

    // ensure that we don't lint inside let-bodies
    // if this primitive is a let-body, we have already linted it
    if LetIn::cast(parent_node).is_some() {
        return None;
    };

    let inner_node = paren.expr()?;
    let parsed_inner = Expr::cast(inner_node.syntax().clone())?;
    if !matches!(
        parsed_inner,
        Expr::List(_)
            | Expr::Paren(_)
            | Expr::Str(_)
            | Expr::AttrSet(_)
            | Expr::Select(_)
            | Expr::Ident(_)
    ) {
        return None;
    };

    let at = paren_expr_range;
    let message = "Useless parentheses around primitive expression";
    let replacement = parsed_inner.syntax().clone();
    Some(Diagnostic::suggest(
        at,
        message,
        Suggestion::new(at, replacement),
    ))
}
