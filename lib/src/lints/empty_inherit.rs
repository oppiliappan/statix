use crate::{Metadata, Report, Rule, Suggestion, session::SessionInfo, utils};

use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind, ast::Inherit};
use rowan::ast::AstNode;

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
        if let NodeOrToken::Node(node) = node
            && let Some(inherit_stmt) = Inherit::cast(node.clone())
            && inherit_stmt.from().is_none()
            && inherit_stmt.attrs().count() == 0
        {
            let at = node.text_range();
            let replacement_at = utils::with_preceeding_whitespace(node);
            let message = "Remove this empty `inherit` statement";
            Some(self.report().suggest(
                at,
                message,
                Suggestion::new(replacement_at, None::<SyntaxElement>),
            ))
        } else {
            None
        }
    }
}
