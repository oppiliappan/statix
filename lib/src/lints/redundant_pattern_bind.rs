use crate::{session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use if_chain::if_chain;
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
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(pattern) = Pattern::cast(node.clone());
            // no patterns within `{ }`
            if pattern.pat_entries().count() == 0;

            // pattern is just ellipsis
            if pattern.ellipsis_token().is_some();

            // pattern is bound
            if let Some(ident) =  pattern.at_token();
            then {
                let at = node.text_range();
                let message = format!("This pattern bind is redundant, use `{}` instead", ident.to_string());
                let replacement = ident.clone();
                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}
