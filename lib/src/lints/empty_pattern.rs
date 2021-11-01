use crate::{make, Metadata, Report, Rule, Suggestion};

use if_chain::if_chain;
use macros::lint;
use rnix::{
    types::{Pattern, TypedNode},
    NodeOrToken, SyntaxElement, SyntaxKind,
};

/// ## What it does
/// Checks for an empty variadic pattern: `{...}`, in a function
/// argument.
///
/// ## Why is this bad?
/// The intention with empty patterns is not instantly obvious. Prefer
/// an underscore identifier instead, to indicate that the argument
/// is being ignored.
///
/// ## Example
///
/// ```nix
/// client = { ... }: {
///   imports = [ self.nixosModules.irmaseal-pkg ];
///   services.irmaseal-pkg.enable = true;
/// };
/// ```
///
/// Replace the empty variadic pattern with `_` to indicate that you
/// intend to ignore the argument:
///
/// ```nix
/// client = _: {
///   imports = [ self.nixosModules.irmaseal-pkg ];
///   services.irmaseal-pkg.enable = true;
/// };
/// ```
#[lint(
    name = "empty pattern",
    note = "Found empty pattern in function argument",
    code = 10,
    match_with = SyntaxKind::NODE_PATTERN
)]
struct EmptyPattern;

impl Rule for EmptyPattern {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(pattern) = Pattern::cast(node.clone());
            // no patterns within `{ }`
            if pattern.entries().count() == 0;
            // pattern is not bound
            if pattern.at().is_none();
            then {
                let at = node.text_range();
                let message = "This pattern is empty, use `_` instead";
                let replacement = make::ident("_").node().clone();
                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}
