use crate::{
    Metadata, Report, Rule, Suggestion, make,
    session::{SessionInfo, Version},
};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    types::{Select, TypedNode},
};

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
        if sess.version() >=  &lint_version
            && let NodeOrToken::Node(node) = node
            && let Some(select_expr) = Select::cast(node.clone())
            && let Some(select_from) = select_expr.set()
            && let Some(group_by_attr) = select_expr.index()
            // a heuristic to lint on nixpkgs.lib.groupBy
            // and lib.groupBy and its variants
            && select_from.text() != "builtins"
            && group_by_attr.text() == "groupBy"
        {
            let at = node.text_range();
            let replacement = {
                let builtins = make::ident("builtins");
                make::select(builtins.node(), &group_by_attr).node().clone()
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
