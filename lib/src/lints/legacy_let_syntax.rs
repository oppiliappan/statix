use crate::{make, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{EntryHolder, Ident, Key, LegacyLet, TokenWrapper, TypedNode},
    NodeOrToken, SyntaxElement, SyntaxKind,
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
    name = "legacy let syntax",
    note = "Using undocumented `let` syntax",
    code = 5,
    match_with = SyntaxKind::NODE_LEGACY_LET
)]
struct ManualInherit;

impl Rule for ManualInherit {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(legacy_let) = LegacyLet::cast(node.clone());

            if legacy_let
                .entries()
                .any(|kv| matches!(kv.key(), Some(k) if key_is_ident(&k, "body")));

            then {
                let inherits = legacy_let.inherits();
                let entries = legacy_let.entries();
                let attrset = make::attrset(inherits, entries, true);
                let parenthesized = make::parenthesize(attrset.node());
                let selected = make::select(parenthesized.node(), make::ident("body").node());

                let at = node.text_range();
                let message = "Prefer `rec` over undocumented `let` syntax";
                let replacement = selected.node().clone();

                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}

fn key_is_ident(key_path: &Key, ident: &str) -> bool {
    if let Some(key_node) = key_path.path().next() {
        if let Some(key) = Ident::cast(key_node) {
            return key.as_str() == ident;
        }
    }
    false
}
