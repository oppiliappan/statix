use crate::{
    Metadata, Report, Rule, Suggestion, make,
    session::{SessionInfo, Version},
};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind, ast::Select};

/// ## What it does
/// Checks for `lib.zipAttrsWith`.
///
/// ## Why is this bad?
/// Nix 2.6 introduces `builtins.zipAttrsWith` which is faster and does
/// not require a lib import.
///
/// ## Example
///
/// ```nix
/// lib.zipAttrsWith (name: values: values) [ {a = "x";} {a = "y"; b = "z";} ]
/// # { a = ["x" "y"]; b = ["z"] }
/// ```
///
/// Replace `lib.zipAttrsWith` with `builtins.zipAttrsWith`:
///
/// ```nix
/// builtins.zipAttrsWith (name: values: values) [ {a = "x";} {a = "y"; b = "z";} ]
/// ```
#[lint(
    name = "faster_zipattrswith",
    note = "Found lib.zipAttrsWith",
    code = 16,
    match_with = SyntaxKind::NODE_SELECT
)]
struct FasterZipAttrsWith;

impl Rule for FasterZipAttrsWith {
    fn validate(&self, node: &SyntaxElement, sess: &SessionInfo) -> Option<Report> {
        let lint_version = "2.6".parse::<Version>().unwrap();
        if_chain! {
            if sess.version() >= &lint_version;
            if let NodeOrToken::Node(node) = node;
            if let Some(select_expr) = Select::cast(node.clone());
            if let Some(select_from) = select_expr.expr();
            if let Some(zip_attrs_with) = select_expr.attrpath();

            let select_from_text = select_from.syntax().text().to_string();
            let zip_prefix = zip_attrs_with
                .attrs()
                .take(zip_attrs_with.attrs().count() - 1)
                .map(|attr| attr.to_string())
                .collect::<Vec<_>>();
            let zip_from = std::iter::once(select_from_text)
                .chain(zip_prefix)
                .collect::<Vec<_>>()
                .join(".");
            // a heuristic to lint on nixpkgs.lib.zipAttrsWith
            // and lib.zipAttrsWith and its variants
            if zip_from != "builtins";
            if zip_attrs_with.syntax()
                .text()
                .to_string()
                .ends_with("zipAttrsWith");

            then {
                let at = node.text_range();
                let replacement = {
                    let builtins = make::ident("builtins");
                    make::select(builtins.syntax(), &zip_attrs_with.syntax())
                };
                let message = format!("Prefer `builtins.zipAttrsWith` over `{}.zipAttrsWith`", zip_from);
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
