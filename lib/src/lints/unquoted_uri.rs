use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};
use rowan::ast::AstNode as _;

use macros::lint;
use rnix::{NodeOrToken, SyntaxElement, SyntaxKind};

/// ## What it does
/// Checks for URI expressions that are not quoted.
///
/// ## Why is this bad?
/// The Nix language has a special syntax for URLs even though quoted
/// strings can also be used to represent them. Unlike paths, URLs do
/// not have any special properties in the Nix expression language
/// that would make the difference useful. Moreover, using variable
/// expansion in URLs requires some URLs to be quoted strings anyway.
/// So the most consistent approach is to always use quoted strings to
/// represent URLs. Additionally, a semicolon immediately after the
/// URL can be mistaken for a part of URL by language-agnostic tools
/// such as terminal emulators.
///
/// See RFC 00045 [1] for more.
///
/// [1]: https://github.com/NixOS/rfcs/blob/master/rfcs/0045-deprecate-url-syntax.md
///
/// ## Example
///
/// ```nix
/// inputs = {
///   gitignore.url = github:hercules-ci/gitignore.nix;
/// }
/// ```
///
/// Quote the URI expression:
///
/// ```nix
/// inputs = {
///   gitignore.url = "github:hercules-ci/gitignore.nix";
/// }
/// ```
#[lint(
    name = "unquoted_uri",
    note = "Found unquoted URI expression",
    code = 12,
    match_with = SyntaxKind::TOKEN_URI
)]
struct UnquotedUri;

impl Rule for UnquotedUri {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Token(token) = node else {
            return None;
        };

        let parent_node = token.parent()?;
        let at = token.text_range();
        let replacement = make::quote(&parent_node);
        let message = "Consider quoting this URI expression";
        Some(self.report().suggest(
            at,
            message,
            Suggestion::with_replacement(at, replacement.syntax().clone()),
        ))
    }
}
