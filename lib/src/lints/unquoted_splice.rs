use crate::{make, session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rowan::ast::AstNode;

use macros::lint;
use rnix::{types::Dynamic, NodeOrToken, SyntaxElement, SyntaxKind};

/// ## What it does
/// Checks for antiquote/splice expressions that are not quoted.
///
/// ## Why is this bad?
/// An *anti*quoted expression should always occur within a *quoted*
/// expression.
///
/// ## Example
///
/// ```nix
/// let
///   pkgs = nixpkgs.legacyPackages.${system};
/// in
///   pkgs
/// ```
///
/// Quote the splice expression:
///
/// ```nix
/// let
///   pkgs = nixpkgs.legacyPackages."${system}";
/// in
///   pkgs
/// ```
#[lint(
    name = "unquoted_splice",
    note = "Found unquoted splice expression",
    code = 9,
    match_with = SyntaxKind::NODE_DYNAMIC
)]
struct UnquotedSplice;

impl Rule for UnquotedSplice {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if let NodeOrToken::Node(node) = node
            && Dynamic::cast(node.clone()).is_some()
        {
            let at = node.text_range();
            let replacement = make::quote(node);
            let message = "Consider quoting this splice expression";
            Some(self.report().suggest(
                at,
                message,
                Suggestion::new(at, replacement.syntax().clone()),
            ))
        } else {
            None
        }
    }
}
