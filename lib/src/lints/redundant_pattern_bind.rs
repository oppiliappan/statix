use crate::{Metadata, Report, Rule, Suggestion, session::SessionInfo};

use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind, ast::Pattern};
use rowan::ast::AstNode;

/// ## What it does
/// Checks for binds of the form `inputs @ { ... }` in function
/// arguments.
///
/// ## Why is this bad?
/// The variadic pattern here is redundant, as it does not capture
/// anything.
///
/// ## Example
///
/// ```nix
/// inputs @ { ... }: inputs.nixpkgs
/// ```
///
/// Remove the pattern altogether:
///
/// ```nix
/// inputs: inputs.nixpkgs
/// ```
#[lint(
    name = "redundant_pattern_bind",
    note = "Found redundant pattern bind in function argument",
    code = 11,
    match_with = SyntaxKind::NODE_PATTERN
)]
struct RedundantPatternBind;

impl Rule for RedundantPatternBind {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if let NodeOrToken::Node(node) = node
            && let Some(pattern) = Pattern::cast(node.clone())
            // no patterns within `{ }`
            && pattern.pat_entries().count() == 0
            // pattern is just ellipsis
            && pattern.ellipsis_token().is_some()
            // pattern is bound
            && let Some(ident) =  pattern.pat_bind().and_then(|bind| bind.ident())
        {
            let at = node.text_range();
            let message = format!(
                "This pattern bind is redundant, use `{ident}` instead"
            );
            let replacement = ident.syntax().clone();
            Some(
                self.report()
                    .suggest(at, message, Suggestion::new(at, Some(replacement))),
            )
        } else {
            None
        }
    }
}
