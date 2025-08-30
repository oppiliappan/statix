use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    types::{EntryHolder, Ident, LegacyLet, TokenWrapper, TypedNode},
};

/// ## What it does
/// Checks for legacy-let syntax that was never formalized.
///
/// ## Why is this bad?
/// This syntax construct is undocumented, refrain from using it.
///
/// ## Example
///
/// Legacy let syntax makes use of an attribute set annotated with
/// `let` and expects a `body` attribute.
/// ```nix
/// let {
///   body = x + y;
///   x = 2;
///   y = 3;
/// }
/// ```
///
/// This is trivially representible via `rec`, which is documented
/// and more widely known:
///
/// ```nix
/// rec {
///   body = x + y;
///   x = 2;
///   y = 3;
/// }.body
/// ```
#[lint(
    name = "legacy_let_syntax",
    note = "Using undocumented `let` syntax",
    code = 5,
    match_with = SyntaxKind::NODE_LEGACY_LET
)]
struct ManualInherit;

impl Rule for ManualInherit {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let legacy_let = LegacyLet::cast(node.clone())?;

        if !legacy_let
            .entries()
            .filter_map(|kv| {
                let key = kv.key()?;
                let first_component = key.path().next()?;
                Ident::cast(first_component)
            })
            .any(|ident| ident.as_str() == "body")
        {
            return None;
        }

        let inherits = legacy_let.inherits();
        let entries = legacy_let.entries();
        let attrset = make::attrset(inherits, entries, true);
        let parenthesized = make::parenthesize(attrset.node());
        let selected = make::select(parenthesized.node(), make::ident("body").node());

        let at = node.text_range();
        let message = "Prefer `rec` over undocumented `let` syntax";
        let replacement = selected.node().clone();

        Some(
            self.report()
                .suggest(at, message, Suggestion::with_replacement(at, replacement)),
        )
    }
}
