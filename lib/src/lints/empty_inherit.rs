use crate::{Metadata, Report, Rule, Suggestion, utils};

use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind, ast::Inherit};
use rowan::ast::AstNode as _;

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
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let inherit_stmt = Inherit::cast(node.clone())?;

        if inherit_stmt.from().is_some() {
            return None;
        }

        if inherit_stmt.attrs().count() != 0 {
            return None;
        }

        let at = node.text_range();
        let replacement_at = utils::with_preceeding_whitespace(node);
        let message = "Remove this empty `inherit` statement";
        Some(
            self.report()
                .suggest(at, message, Suggestion::with_empty(replacement_at)),
        )
    }
}
