use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    ast::{AttrSet, Entry, HasEntry as _, Lambda, Param},
};
use rowan::ast::AstNode as _;

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
///   services.irmaseal-pkg.enable = true;
/// };
/// ```
///
/// Replace the empty variadic pattern with `_` to indicate that you
/// intend to ignore the argument:
///
/// ```nix
/// client = _: {
///   services.irmaseal-pkg.enable = true;
/// };
/// ```
#[lint(
    name = "empty_pattern",
    note = "Found empty pattern in function argument",
    code = 10,
    match_with = SyntaxKind::NODE_LAMBDA
)]
struct EmptyPattern;

impl Rule for EmptyPattern {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let lambda_expr = Lambda::cast(node.clone())?;

        let Some(Param::Pattern(pattern)) = lambda_expr.param() else {
            return None;
        };

        // no patterns within `{ }`
        if pattern.pat_entries().count() != 0 {
            return None;
        }

        // pattern is not bound
        if pattern.pat_bind().is_some() {
            return None;
        }

        if is_module(lambda_expr.body()?.syntax()) {
            return None;
        }

        Some(self.report().suggest(
            pattern.syntax().text_range(),
            "This pattern is empty, use `_` instead",
            Suggestion::with_replacement(
                pattern.syntax().text_range(),
                make::ident("_").syntax().clone(),
            ),
        ))
    }
}

fn is_module(body: &SyntaxNode) -> bool {
    let Some(attr_set) = AttrSet::cast(body.clone()) else {
        return false;
    };

    attr_set
        .entries()
        .filter_map(|e| {
            let Entry::AttrpathValue(attrpath_value) = e else {
                return None;
            };

            attrpath_value.attrpath()
        })
        .any(|k| k.to_string() == "imports")
}
