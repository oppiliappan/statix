use crate::{make, session::SessionInfo, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{BinOp, BinOpKind, IfElse, Select, TypedNode},
    NodeOrToken, SyntaxElement, SyntaxKind,
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
            if let Some(cond_bin_expr) = BinOp::cast(condition_expr.clone());
            if let Some(BinOpKind::IsSet) = cond_bin_expr.operator();

            // set ? attr_path
            // ^^^--------------- lhs
            //      ^^^^^^^^^^--- rhs
            if let Some(set) = cond_bin_expr.lhs();
            if let Some(attr_path) = cond_bin_expr.rhs();

            // check if body of the `if` expression is of the form `set.attr_path`
            if let Some(body_expr) = if_else_expr.body();
            if let Some(body_select_expr) = Select::cast(body_expr.clone());
            let expected_body = make::select(&set, &attr_path);

            // text comparison will do for now
            if body_select_expr.node().text() == expected_body.node().text();
            then {
                let at = node.text_range();
                // `or` is tightly binding, we need to parenthesize non-literal exprs
                let default_with_parens = match default_expr.kind() {
                    SyntaxKind::NODE_LIST
                    | SyntaxKind::NODE_PAREN
                    | SyntaxKind::NODE_STRING
                    | SyntaxKind::NODE_ATTR_SET
                    | SyntaxKind::NODE_IDENT
                    | SyntaxKind::NODE_SELECT => default_expr,
                    _ => make::parenthesize(&default_expr).node().clone(),
                };
                let replacement = make::or_default(&set, &attr_path, &default_with_parens).node().clone();
                let message = format!(
                    "Consider using `{}` instead of this `if` expression",
                    replacement
                );
                Some(
                    self.report()
                        .suggest(at, message, Suggestion::new(at, replacement)),
                )
            } else {
                None
            }
        }
    }
}
