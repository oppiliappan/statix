use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{HasAttr, IfElse, Select},
};

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
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(if_else_expr) = IfElse::cast(node.clone());
            if let Some(condition_expr) = if_else_expr.condition();
            if let Some(default_expr) = if_else_expr.else_body();
            if let Some(cond_bin_expr) = HasAttr::cast(condition_expr.syntax().clone());

            // set ? attr_path
            // ^^^--------------- lhs
            //      ^^^^^^^^^^--- rhs
            if let Some(set) = cond_bin_expr.expr();
            if let Some(attr_path) = cond_bin_expr.attrpath();

            // check if body of the `if` expression is of the form `set.attr_path`
            if let Some(body_expr) = if_else_expr.body();
            if let Some(body_select_expr) = Select::cast(body_expr.syntax().clone());
            let expected_body = make::select(&set.syntax(), &attr_path.syntax());

            // text comparison will do for now
            if body_select_expr.syntax().text() == expected_body.syntax().text();
            then {
                let at = node.text_range();
                // `or` is tightly binding, we need to parenthesize non-literal exprs
                let default_with_parens = match default_expr {
                    rnix::ast::Expr::List(_)
                    | rnix::ast::Expr::Paren(_)
                    | rnix::ast::Expr::Str(_)
                    | rnix::ast::Expr::AttrSet(_)
                    | rnix::ast::Expr::Ident(_)
                    | rnix::ast::Expr::Select(_) => default_expr.syntax().clone(),
                    _ => make::parenthesize(default_expr.syntax()).syntax().clone(),
                };
                let replacement = make::or_default(&set.syntax(), &attr_path.syntax(), &default_with_parens);
                let message = format!(
                    "Consider using `{}` instead of this `if` expression",
                    replacement
                );
                Some(
                    self.report()
                        .suggest(at, message, Suggestion::new(at, replacement.syntax().clone())),
                )
            } else {
                None
            }
        }
    }
}
