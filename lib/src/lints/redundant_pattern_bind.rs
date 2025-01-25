use crate::{session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use macros::lint;
use rnix::{ast::Pattern, NodeOrToken, SyntaxElement, SyntaxKind};

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
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        let pattern = Pattern::cast(node.clone())?;
        // no patterns within `{ }`
        if pattern.pat_entries().count() != 0 {
            return None;
        };

        // pattern is just ellipsis
        pattern.ellipsis_token()?;

        // pattern is bound
        let ident = pattern.pat_bind().and_then(|bind| bind.ident())?;

        let at = node.text_range();
        let message = format!("This pattern bind is redundant, use `{}` instead", ident);
        let replacement = ident.clone();
        Some(self.report().suggest(
            at,
            message,
            Suggestion::new(at, replacement.syntax().clone()),
        ))
    }
}
