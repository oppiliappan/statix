use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo, utils};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind, ast::Inherit};

/// ## What it does
/// Checks for empty inherit statements.
///
/// ## Why is this bad?
/// Useless code, probably the result of a refactor.
///
/// ## Example
///
/// ```nix
/// inherit;
/// ```
///
/// Remove it altogether.
#[lint(
    name = "empty_inherit",
    note = "Found empty inherit statement",
    code = 14,
    match_with = SyntaxKind::NODE_INHERIT
)]
struct EmptyInherit;

impl Rule for EmptyInherit {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(inherit_stmt) = Inherit::cast(node.clone());
            if inherit_stmt.from().is_none();
            if inherit_stmt.attrs().count() == 0;
            then {
                let at = node.text_range();
                let replacement = make::empty();
                let replacement_at = utils::with_preceeding_whitespace(node);
                let message = "Remove this empty `inherit` statement";
                Some(
                    self
                    .report()
                    .suggest(at, message, Suggestion::new(replacement_at, replacement.syntax().clone()))
                )
            } else {
                None
            }
        }
    }
}
