use crate::{
    make,
    session::{SessionInfo, Version},
    Metadata, Report, Rule, Suggestion,
};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{ast::Select, NodeOrToken, SyntaxElement, SyntaxKind};

/// ## What it does
/// Checks for `lib.groupBy`.
///
/// ## Why is this bad?
/// Nix 2.5 introduces `builtins.groupBy` which is faster and does
/// not require a lib import.
///
/// ## Example
///
/// ```nix
/// lib.groupBy (x: if x > 2 then "big" else "small") [ 1 2 3 4 5 6 ];
/// # { big = [ 3 4 5 6 ]; small = [ 1 2 ]; }
/// ```
///
/// Replace `lib.groupBy` with `builtins.groupBy`:
///
/// ```nix
/// builtins.groupBy (x: if x > 2 then "big" else "small") [ 1 2 3 4 5 6 ];
/// ```
#[lint(
    name = "faster_groupby",
    note = "Found lib.groupBy",
    code = 15,
    match_with = SyntaxKind::NODE_SELECT
)]
struct FasterGroupBy;

impl Rule for FasterGroupBy {
    fn validate(&self, node: &SyntaxElement, sess: &SessionInfo) -> Option<Report> {
        let lint_version = "2.5".parse::<Version>().unwrap();
        if_chain! {
            if sess.version() >=  &lint_version;
            if let NodeOrToken::Node(node) = node;
            if let Some(select_expr) = Select::cast(node.clone());
            if let Some(select_from) = select_expr.expr();
            if let Some(group_by_attr) = select_expr.attrpath();

            // a heuristic to lint on nixpkgs.lib.groupBy
            // and lib.groupBy and its variants
            if select_from.syntax().text() != "builtins";
            if group_by_attr.syntax().text() == "groupBy";

            then {
                let at = node.text_range();
                let replacement = {
                    let builtins = make::ident("builtins");
                    make::select(builtins.syntax(), group_by_attr.syntax()).syntax().clone()
                };
                let message = format!("Prefer `builtins.groupBy` over `{}.groupBy`", select_from);
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
