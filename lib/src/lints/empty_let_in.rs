use crate::{Metadata, Report, Rule, Suggestion, session::SessionInfo};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{HasEntry, LetIn},
};

/// ## What it does
/// Checks for `let-in` expressions which create no new bindings.
///
/// ## Why is this bad?
/// `let-in` expressions that create no new bindings are useless.
/// These are probably remnants from debugging or editing expressions.
///
/// ## Example
///
/// ```nix
/// let in pkgs.statix
/// ```
///
/// Preserve only the body of the `let-in` expression:
///
/// ```nix
/// pkgs.statix
/// ```
#[lint(
    name = "empty_let_in",
    note = "Useless let-in expression",
    code = 2,
    match_with = SyntaxKind::NODE_LET_IN
)]
struct EmptyLetIn;

impl Rule for EmptyLetIn {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(let_in_expr) = LetIn::cast(node.clone());
            let entries = let_in_expr.entries();
            let inherits = let_in_expr.inherits();

            if entries.count() == 0;
            if inherits.count() == 0;

            if let Some(body) = let_in_expr.body();

            // ensure that the let-in-expr does not have comments
            let has_comments = node
                .children_with_tokens()
                .any(|el| el.kind() == SyntaxKind::TOKEN_COMMENT);
            then {
                let at = node.text_range();
                let replacement = body;
                let message = "This let-in expression has no entries";
                Some(if has_comments {
                    self.report().diagnostic(at, message)
                } else {
                    self.report().suggest(at, message, Suggestion::new(at, replacement.syntax().clone()))
                })
            } else {
                None
            }
        }
    }
}
