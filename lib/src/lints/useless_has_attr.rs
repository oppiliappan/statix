use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{Expr, IfElse},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks for expressions that use the "has attribute" operator: `?`,
/// where the `or` operator would suffice.
///
/// ## Why is this bad?
/// The `or` operator is more readable.
///
/// ## Example
/// ```nix
/// if x ? a then x.a else some_default
/// ```
///
/// Use `or` instead:
///
/// ```nix
/// x.a or some_default
/// ```
#[lint(
    name = "useless_has_attr",
    note = "This `if` expression can be simplified with `or`",
    code = 19,
    match_with = SyntaxKind::NODE_IF_ELSE
)]
struct UselessHasAttr;

impl Rule for UselessHasAttr {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let if_else_expr = IfElse::cast(node.clone())?;
        let condition_expr = if_else_expr.condition()?;
        let default_expr = if_else_expr.else_body()?;

        let Expr::HasAttr(has_attr) = condition_expr else {
            return None;
        };

        // set ? attr_path
        let set = has_attr.expr()?;
        let attr_path = has_attr.attrpath()?;

        // check if body of the `if` expression is of the form `set.attr_path`
        let body_expr = if_else_expr.body()?;
        let Expr::Select(body_select_expr) = body_expr else {
            return None;
        };

        let expected_body = make::select(set.syntax(), attr_path.syntax());

        // text comparison will do for now
        if body_select_expr.syntax().text() != expected_body.syntax().text() {
            return None;
        }

        let at = node.text_range();

        // `or` is tightly binding, we need to parenthesize non-literal exprs
        let default_with_parens = match default_expr {
            Expr::List(_)
            | Expr::Paren(_)
            | Expr::Str(_)
            | Expr::AttrSet(_)
            | Expr::Ident(_)
            | Expr::Select(_) => default_expr,
            _ => Expr::Paren(make::parenthesize(default_expr.syntax())),
        };

        let replacement = make::or_default(
            set.syntax(),
            attr_path.syntax(),
            default_with_parens.syntax(),
        )
        .syntax()
        .clone();
        let message = format!("Consider using `{replacement}` instead of this `if` expression");
        Some(
            self.report()
                .suggest(at, message, Suggestion::with_replacement(at, replacement)),
        )
    }
}
